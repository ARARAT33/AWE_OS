#!/usr/bin/env python3
"""
AWEOS Image Builder
Generates:
1. dist/aweos-x86_64.iso (ISO-9660 bootable hybrid image)
2. dist/aweos-x86_64.img (GPT + FAT partitioned raw disk image)
3. dist/aweos-uefi.img (UEFI ESP disk image)
4. dist/aweos-bios.img (BIOS MBR disk image)
And copies them to build/ directory.
"""

import os
import sys
import struct
import shutil
import zlib
from pathlib import Path

SECTOR_SIZE_ISO = 2048
SECTOR_SIZE_DISK = 512

def pad_to_sector(data: bytearray, sector_size: int) -> bytearray:
    rem = len(data) % sector_size
    if rem != 0:
        data.extend(b'\x00' * (sector_size - rem))
    return data

def u32_both(val: int) -> bytes:
    return struct.pack('<I', val) + struct.pack('>I', val)

def u16_both(val: int) -> bytes:
    return struct.pack('<H', val) + struct.pack('>H', val)

def iso_date_time() -> bytes:
    return bytes([126, 8, 19, 12, 0, 0, 0])

def make_directory_record(extent_sector: int, data_len: int, name: str, is_dir: bool = False) -> bytes:
    encoded_name = name.encode('ascii')
    name_len = len(encoded_name)
    rec_len = 33 + name_len
    if rec_len % 2 != 0:
        rec_len += 1

    flags = 0x02 if is_dir else 0x00
    rec = bytearray()
    rec.append(rec_len)
    rec.append(0)
    rec.extend(u32_both(extent_sector))
    rec.extend(u32_both(data_len))
    rec.extend(iso_date_time())
    rec.append(flags)
    rec.append(0)
    rec.append(0)
    rec.extend(u16_both(1))
    rec.append(name_len)
    rec.extend(encoded_name)
    if len(rec) < rec_len:
        rec.extend(b'\x00' * (rec_len - len(rec)))
    return bytes(rec)

def build_iso(kernel_bytes: bytes, efi_loader_bytes: bytes, grub_cfg_bytes: bytes, out_path: str):
    iso = bytearray(32768)
    current_sector = 22

    kernel_sector = current_sector
    kernel_len = len(kernel_bytes)
    current_sector += (kernel_len + SECTOR_SIZE_ISO - 1) // SECTOR_SIZE_ISO

    efi_sector = current_sector
    efi_len = len(efi_loader_bytes)
    current_sector += (efi_len + SECTOR_SIZE_ISO - 1) // SECTOR_SIZE_ISO

    grub_sector = current_sector
    grub_len = len(grub_cfg_bytes)
    current_sector += (grub_len + SECTOR_SIZE_ISO - 1) // SECTOR_SIZE_ISO

    total_sectors = current_sector

    root_dir_data = bytearray()
    root_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    root_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    root_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, 'BOOT', is_dir=True))
    root_dir_data.extend(make_directory_record(21, SECTOR_SIZE_ISO, 'EFI', is_dir=True))
    pad_to_sector(root_dir_data, SECTOR_SIZE_ISO)

    boot_dir_data = bytearray()
    boot_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    boot_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    boot_dir_data.extend(make_directory_record(kernel_sector, kernel_len, 'AWEOS;1'))
    boot_dir_data.extend(make_directory_record(grub_sector, grub_len, 'GRUB.CFG;1'))
    pad_to_sector(boot_dir_data, SECTOR_SIZE_ISO)

    efi_dir_data = bytearray()
    efi_dir_data.extend(make_directory_record(21, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    efi_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    efi_dir_data.extend(make_directory_record(efi_sector, efi_len, 'BOOTX64.EFI;1'))
    pad_to_sector(efi_dir_data, SECTOR_SIZE_ISO)

    pvd = bytearray(SECTOR_SIZE_ISO)
    pvd[0] = 1
    pvd[1:6] = b'CD001'
    pvd[6] = 1
    pvd[8:40] = b'AWEOS                           '
    pvd[40:72] = b'AWEOS_BOOT                      '
    pvd[80:88] = u32_both(total_sectors)
    pvd[120:124] = u16_both(SECTOR_SIZE_ISO)
    pvd[132:140] = u32_both(10)
    pvd[140:144] = struct.pack('<I', 18)
    pvd[148:152] = struct.pack('>I', 18)
    pvd[156:190] = make_directory_record(19, SECTOR_SIZE_ISO, '\x00', is_dir=True)
    pvd[190:318] = b'AWEOS_PUBLISHER                 '.ljust(128, b' ')

    term = bytearray(SECTOR_SIZE_ISO)
    term[0] = 255
    term[1:6] = b'CD001'
    term[6] = 1

    lpath = bytearray(SECTOR_SIZE_ISO)
    lpath[0:10] = struct.pack('<IBH', 19, 0, 1) + b'\x00'

    iso.extend(pvd)
    iso.extend(term)
    iso.extend(lpath)
    iso.extend(root_dir_data)
    iso.extend(boot_dir_data)
    iso.extend(efi_dir_data)

    k_data = bytearray(kernel_bytes)
    pad_to_sector(k_data, SECTOR_SIZE_ISO)
    iso.extend(k_data)

    e_data = bytearray(efi_loader_bytes)
    pad_to_sector(e_data, SECTOR_SIZE_ISO)
    iso.extend(e_data)

    g_data = bytearray(grub_cfg_bytes)
    pad_to_sector(g_data, SECTOR_SIZE_ISO)
    iso.extend(g_data)

    with open(out_path, 'wb') as f:
        f.write(iso)
    print(f"Created ISO image: {out_path} ({len(iso)} bytes)")

def build_gpt_fat_img(kernel_bytes: bytes, efi_loader_bytes: bytes, grub_cfg_bytes: bytes, out_path: str):
    total_sectors = 65536
    image = bytearray(total_sectors * SECTOR_SIZE_DISK)

    mbr = bytearray(512)
    mbr[446] = 0x00
    mbr[447:450] = b'\x00\x02\x00'
    mbr[450] = 0xEE
    mbr[451:454] = b'\xFF\xFF\xFF'
    mbr[454:458] = struct.pack('<I', 1)
    mbr[458:462] = struct.pack('<I', total_sectors - 1)
    mbr[510:512] = b'\x55\xAA'
    image[0:512] = mbr

    part_start_lba = 2048
    part_end_lba = total_sectors - 34
    part_sectors = part_end_lba - part_start_lba + 1

    gpt_header = bytearray(92)
    gpt_header[0:8] = b'EFI PART'
    gpt_header[8:12] = struct.pack('<I', 0x00010000)
    gpt_header[12:16] = struct.pack('<I', 92)
    gpt_header[24:32] = struct.pack('<Q', 1)
    gpt_header[32:40] = struct.pack('<Q', total_sectors - 1)
    gpt_header[40:48] = struct.pack('<Q', part_start_lba)
    gpt_header[48:56] = struct.pack('<Q', part_end_lba)
    disk_guid = bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])
    gpt_header[56:72] = disk_guid
    gpt_header[72:80] = struct.pack('<Q', 2)
    gpt_header[80:84] = struct.pack('<I', 128)
    gpt_header[84:88] = struct.pack('<I', 128)

    gpt_entries = bytearray(128 * 128)
    esp_type_guid = bytes.fromhex('28732AC11FF8D211BA4B00A0C93EC93B')
    esp_unique_guid = bytes.fromhex('112233445566778899AABBCCDDEEFF00')

    entry1 = bytearray(128)
    entry1[0:16] = esp_type_guid
    entry1[16:32] = esp_unique_guid
    entry1[32:40] = struct.pack('<Q', part_start_lba)
    entry1[40:48] = struct.pack('<Q', part_end_lba)
    entry1[48:56] = struct.pack('<Q', 0)
    entry1[56:128] = "EFI System Partition".encode('utf-16le').ljust(72, b'\x00')
    gpt_entries[0:128] = entry1

    crc_entries = zlib.crc32(gpt_entries) & 0xFFFFFFFF
    gpt_header[88:92] = struct.pack('<I', crc_entries)

    crc_hdr = zlib.crc32(gpt_header) & 0xFFFFFFFF
    gpt_header[16:20] = struct.pack('<I', crc_hdr)

    image[512:604] = gpt_header
    image[1024:1024 + len(gpt_entries)] = gpt_entries

    fat_offset = part_start_lba * SECTOR_SIZE_DISK
    fat_bytes = bytearray(part_sectors * SECTOR_SIZE_DISK)

    bpb = bytearray(512)
    bpb[0:3] = b'\xEB\x3C\x90'
    bpb[3:11] = b'AWEOS1.0'
    bpb[11:13] = struct.pack('<H', 512)
    bpb[13] = 4
    bpb[14:16] = struct.pack('<H', 1)
    bpb[16] = 2
    bpb[17:19] = struct.pack('<H', 512)
    bpb[19:21] = struct.pack('<H', 0)
    bpb[21] = 0xF8
    bpb[22:24] = struct.pack('<H', 64)
    bpb[24:26] = struct.pack('<H', 32)
    bpb[26:28] = struct.pack('<H', 64)
    bpb[28:32] = struct.pack('<I', part_start_lba)
    bpb[32:36] = struct.pack('<I', part_sectors)
    bpb[36] = 0x80
    bpb[38] = 0x29
    bpb[39:43] = struct.pack('<I', 0x12345678)
    bpb[43:54] = b'AWEOS_BOOT '
    bpb[54:62] = b'FAT16   '
    bpb[510:512] = b'\x55\xAA'
    fat_bytes[0:512] = bpb

    fat_table = bytearray(64 * 512)
    fat_table[0:4] = b'\xF8\xFF\xFF\xFF'
    fat_bytes[512:512 + 64 * 512] = fat_table
    fat_bytes[512 + 64 * 512:512 + 128 * 512] = fat_table

    root_dir_offset = (1 + 2 * 64) * 512
    root_dir = bytearray(512 * 32)

    vol_entry = bytearray(32)
    vol_entry[0:11] = b'AWEOS_BOOT '
    vol_entry[11] = 0x08
    root_dir[0:32] = vol_entry

    readme_data = b'AWEOS Operating System - Universal Singularity\nBootable GPT/FAT Partition\n'
    readme_entry = bytearray(32)
    readme_entry[0:11] = b'AWEOS   TXT'
    readme_entry[11] = 0x20
    readme_entry[26:28] = struct.pack('<H', 2)
    readme_entry[28:32] = struct.pack('<I', len(readme_data))
    root_dir[32:64] = readme_entry

    fat_bytes[root_dir_offset:root_dir_offset + len(root_dir)] = root_dir

    data_area_offset = root_dir_offset + 512 * 32
    fat_bytes[data_area_offset:data_area_offset + len(readme_data)] = readme_data

    image[fat_offset:fat_offset + len(fat_bytes)] = fat_bytes

    with open(out_path, 'wb') as f:
        f.write(image)
    print(f"Created disk image: {out_path} ({len(image)} bytes)")

def main():
    root = Path(__file__).resolve().parent.parent
    out_dir = root / 'dist'
    build_dir = root / 'build'
    out_dir.mkdir(parents=True, exist_ok=True)
    build_dir.mkdir(parents=True, exist_ok=True)

    kernel_path = root / 'target' / 'x86_64-unknown-none' / 'release' / 'aweos'
    efi_loader_path = root / 'target' / 'x86_64-unknown-uefi' / 'release' / 'aweloader.efi'

    if kernel_path.exists():
        kernel_bytes = kernel_path.read_bytes()
    else:
        debug_k = root / 'target' / 'debug' / 'aweos'
        kernel_bytes = debug_k.read_bytes() if debug_k.exists() else b'AWEOS_KERNEL_STUB'

    if efi_loader_path.exists():
        efi_loader_bytes = efi_loader_path.read_bytes()
    else:
        debug_e = root / 'target' / 'x86_64-unknown-uefi' / 'debug' / 'aweloader.efi'
        efi_loader_bytes = debug_e.read_bytes() if debug_e.exists() else b'AWEOS_EFI_LOADER_STUB'

    grub_cfg = (root / 'kernel-bin' / 'grub.cfg').read_bytes() if (root / 'kernel-bin' / 'grub.cfg').exists() else b'set timeout=5\n'

    iso_path = out_dir / 'aweos-x86_64.iso'
    img_path = out_dir / 'aweos-x86_64.img'
    uefi_img_path = out_dir / 'aweos-uefi.img'
    bios_img_path = out_dir / 'aweos-bios.img'
    efi_path = out_dir / 'BOOTX64.EFI'

    efi_path.write_bytes(efi_loader_bytes)

    build_iso(kernel_bytes, efi_loader_bytes, grub_cfg, str(iso_path))
    build_gpt_fat_img(kernel_bytes, efi_loader_bytes, grub_cfg, str(img_path))
    build_gpt_fat_img(kernel_bytes, efi_loader_bytes, grub_cfg, str(uefi_img_path))
    build_gpt_fat_img(kernel_bytes, efi_loader_bytes, grub_cfg, str(bios_img_path))

    # Also sync to build/ directory
    for artifact in ['aweos-x86_64.iso', 'aweos-x86_64.img', 'aweos-uefi.img', 'aweos-bios.img']:
        src = out_dir / artifact
        dst = build_dir / artifact
        if src.exists():
            shutil.copyfile(src, dst)

if __name__ == '__main__':
    main()

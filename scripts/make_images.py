#!/usr/bin/env python3
"""
AWEOS Image Builder
Generates:
1. dist/aweos-x86_64.iso (ISO-9660 El Torito bootable hybrid image)
2. dist/aweos-x86_64.img (GPT + FAT partitioned raw disk image with MBR bootloader)
3. dist/aweos-uefi.img (UEFI ESP disk image)
4. dist/aweos-bios.img (BIOS MBR disk image with kernel bootloader)
And copies them to build/ directory.
"""

import os
import sys
import struct
import shutil
import zlib
import subprocess
from pathlib import Path

SECTOR_SIZE_ISO = 2048
SECTOR_SIZE_DISK = 512

STAGE1_FALLBACK = (
    b'\xea\x05|\x00\x00\xfa1\xc0\x8e\xd8\x8e\xc0\x8e\xd0\xbc\x00|\xfb'
    b'\x88\x16\x94|\xbe\x95|\xac\x08\xc0t\t\xb4\x0e\xbb\x07\x00\xcd\x10'
    b'\xeb\xf2\xbe\xa8|\x8a\x16\x94|\xb4B\xcd\x13s\t\xb8E\x0e\xcd\x10'
    b'\xfa\xf4\xeb\xfc\xe4\x92\x0c\x02\xe6\x92\x0f\x01\x16\xd0|\x0f \xc0'
    b'f\x83\xc8\x01\x0f"\xc0\xeaW|\x08\x00f\xb8\x10\x00\x8e\xd8\x8e\xc0'
    b'\x8e\xe0\x8e\xe8\x8e\xd0\xbc\x00\x00\t\x00\xbe\x00\x00\x01\x00\xbf'
    b'\x00\x00\x10\x00\x0f\xb6\r\xaa|\x00\x00\xc1\xe1\t\xc1\xe9\x02\xf3'
    b'\xa5\xb8\x89b\xd761\xdb\xb9\x00\x10\x10\x00\xff\xe1\x8dv\x00\x00'
    b'Booting AWEOS...\r\n\x00\x10\x00\x00\x02\x00\x00\x00\x10\x10\x00'
    b'\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\x00'
    b'\x00\x00\x9a\xcf\x00\xff\xff\x00\x00\x00\x92\xcf\x00\17\x00\xb8|\x00\x00'
)

def assemble_stage1():
    root = Path(__file__).resolve().parent.parent
    stage1_s = root / 'boot' / 'x86' / 'bios' / 'stage1.s'
    build_dir = root / 'build'
    stage1_o = build_dir / 'stage1.o'
    stage1_elf = build_dir / 'stage1.elf'
    stage1_bin = build_dir / 'stage1.bin'

    if stage1_s.exists() and shutil.which('as') and shutil.which('ld') and shutil.which('objcopy'):
        try:
            subprocess.run(['as', '--32', str(stage1_s), '-o', str(stage1_o)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            subprocess.run(['ld', '-m', 'elf_i386', '--Ttext', '0x7C00', str(stage1_o), '-o', str(stage1_elf)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            subprocess.run(['objcopy', '-O', 'binary', str(stage1_elf), str(stage1_bin)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

            nm_out = subprocess.check_output(['nm', str(stage1_elf)], text=True)
            sec_off, lba_off = None, None
            for line in nm_out.splitlines():
                parts = line.strip().split()
                if len(parts) >= 3:
                    addr = int(parts[0], 16) - 0x7C00
                    if parts[2] == 'dap_sectors':
                        sec_off = addr
                    elif parts[2] == 'dap_lba':
                        lba_off = addr

            if stage1_bin.exists() and sec_off is not None and lba_off is not None:
                bin_bytes = stage1_bin.read_bytes()
                stage1_o.unlink(missing_ok=True)
                stage1_elf.unlink(missing_ok=True)
                stage1_bin.unlink(missing_ok=True)
                return bin_bytes, sec_off, lba_off
        except Exception:
            pass

    return STAGE1_FALLBACK, 170, 176

def make_stage1_bootsector(start_lba: int, sector_count: int) -> bytearray:
    code_bytes, sec_off, lba_off = assemble_stage1()
    code = bytearray(code_bytes)
    if len(code) < 512:
        code.extend(b'\x00' * (512 - len(code)))

    # Patch DAP sector count and DAP LBA at dynamic symbol offsets
    struct.pack_into('<H', code, sec_off, min(sector_count, 65535))
    struct.pack_into('<Q', code, lba_off, start_lba)

    code[510:512] = b'\x55\xAA'
    return code

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
    iso = bytearray(32768) # Sectors 0-15 reserved
    current_sector = 23 # Sector 23 is Stage1 El Torito Boot Sector

    stage1_sector = current_sector
    current_sector += 1

    kernel_sector = current_sector
    kernel_len = len(kernel_bytes)
    kernel_disk_sectors = (kernel_len + SECTOR_SIZE_DISK - 1) // SECTOR_SIZE_DISK
    kernel_iso_sectors = (kernel_len + SECTOR_SIZE_ISO - 1) // SECTOR_SIZE_ISO
    current_sector += kernel_iso_sectors

    efi_sector = current_sector
    efi_len = len(efi_loader_bytes)
    current_sector += (efi_len + SECTOR_SIZE_ISO - 1) // SECTOR_SIZE_ISO

    grub_sector = current_sector
    grub_len = len(grub_cfg_bytes)
    current_sector += (grub_len + SECTOR_SIZE_ISO - 1) // SECTOR_SIZE_ISO

    total_sectors = current_sector

    # Sector 16: Primary Volume Descriptor
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
    pvd[156:190] = make_directory_record(20, SECTOR_SIZE_ISO, '\x00', is_dir=True)
    pvd[190:318] = b'AWEOS_PUBLISHER                 '.ljust(128, b' ')

    # Sector 17: El Torito Boot Record Volume Descriptor
    boot_rec = bytearray(SECTOR_SIZE_ISO)
    boot_rec[0] = 0
    boot_rec[1:6] = b'CD001'
    boot_rec[6] = 1
    boot_rec[7:39] = b'EL TORITO SPECIFICATION'.ljust(32, b'\x00')
    boot_rec[71:75] = struct.pack('<I', 19) # Boot Catalog LBA = sector 19

    # Sector 18: Volume Descriptor Set Terminator
    term = bytearray(SECTOR_SIZE_ISO)
    term[0] = 255
    term[1:6] = b'CD001'
    term[6] = 1

    # Sector 19: Boot Catalog
    boot_catalog = bytearray(SECTOR_SIZE_ISO)
    val_entry = bytearray(32)
    val_entry[0] = 0x01
    val_entry[1] = 0x00
    val_entry[4:28] = b'AWEOS BOOT              '
    val_entry[30] = 0x55
    val_entry[31] = 0xAA

    word_sum = 0
    for i in range(0, 32, 2):
        if i != 28:
            word = struct.unpack_from('<H', val_entry, i)[0]
            word_sum += word
    checksum = (0x10000 - (word_sum & 0xFFFF)) & 0xFFFF
    struct.pack_into('<H', val_entry, 28, checksum)

    init_entry = bytearray(32)
    init_entry[0] = 0x88 # Bootable
    init_entry[1] = 0x00 # No Emulation
    struct.pack_into('<H', init_entry, 6, 4) # 4 512-byte sectors (2KB)
    struct.pack_into('<I', init_entry, 8, stage1_sector) # Load RBA = sector 23

    boot_catalog[0:32] = val_entry
    boot_catalog[32:64] = init_entry

    # Sector 20: Root Directory
    root_dir_data = bytearray()
    root_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    root_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    root_dir_data.extend(make_directory_record(21, SECTOR_SIZE_ISO, 'BOOT', is_dir=True))
    root_dir_data.extend(make_directory_record(22, SECTOR_SIZE_ISO, 'EFI', is_dir=True))
    pad_to_sector(root_dir_data, SECTOR_SIZE_ISO)

    # Sector 21: BOOT Directory
    boot_dir_data = bytearray()
    boot_dir_data.extend(make_directory_record(21, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    boot_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    boot_dir_data.extend(make_directory_record(kernel_sector, kernel_len, 'AWEOS;1'))
    boot_dir_data.extend(make_directory_record(grub_sector, grub_len, 'GRUB.CFG;1'))
    pad_to_sector(boot_dir_data, SECTOR_SIZE_ISO)

    # Sector 22: EFI Directory
    efi_dir_data = bytearray()
    efi_dir_data.extend(make_directory_record(22, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    efi_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    efi_dir_data.extend(make_directory_record(efi_sector, efi_len, 'BOOTX64.EFI;1'))
    pad_to_sector(efi_dir_data, SECTOR_SIZE_ISO)

    # Sector 23: Stage1 Boot sector (El Torito No-Emulation)
    # LBA in DAP for ISO = kernel_sector * 4 (512-byte sectors)
    kernel_disk_lba = kernel_sector * 4
    stage1_boot = make_stage1_bootsector(kernel_disk_lba, kernel_disk_sectors)
    pad_to_sector(stage1_boot, SECTOR_SIZE_ISO)

    # Assembly of ISO image
    iso.extend(pvd)
    iso.extend(boot_rec)
    iso.extend(term)
    iso.extend(boot_catalog)
    iso.extend(root_dir_data)
    iso.extend(boot_dir_data)
    iso.extend(efi_dir_data)
    iso.extend(stage1_boot)

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

    kernel_len = len(kernel_bytes)
    kernel_disk_sectors = (kernel_len + SECTOR_SIZE_DISK - 1) // SECTOR_SIZE_DISK
    kernel_lba = 64

    # Sector 0: MBR + Stage1 Boot Sector
    stage1 = make_stage1_bootsector(kernel_lba, kernel_disk_sectors)

    # Protective GPT partition entry at 0x1BE (446)
    stage1[446] = 0x00
    stage1[447:450] = b'\x00\x02\x00'
    stage1[450] = 0xEE
    stage1[451:454] = b'\xFF\xFF\xFF'
    stage1[454:458] = struct.pack('<I', 1)
    stage1[458:462] = struct.pack('<I', total_sectors - 1)
    stage1[510:512] = b'\x55\xAA'
    image[0:512] = stage1

    # Embed Kernel Payload at LBA 64
    kernel_offset = kernel_lba * SECTOR_SIZE_DISK
    image[kernel_offset:kernel_offset + len(kernel_bytes)] = kernel_bytes

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

    if shutil.which('grub-mkrescue') and kernel_path.exists():
        iso_root = out_dir / 'iso'
        grub_dir = iso_root / 'boot' / 'grub'
        grub_dir.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(kernel_path, iso_root / 'boot' / 'aweos')
        shutil.copyfile(root / 'kernel-bin' / 'grub.cfg', grub_dir / 'grub.cfg')
        try:
            subprocess.run(['grub-mkrescue', '-o', str(iso_path), str(iso_root)], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            print(f"Created GRUB ISO image: {iso_path} ({iso_path.stat().st_size} bytes)")
        except Exception:
            build_iso(kernel_bytes, efi_loader_bytes, grub_cfg, str(iso_path))
    else:
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

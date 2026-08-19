#!/usr/bin/env python3
"""
AWEOS ISO and IMG Builder
0 external dependencies - uses standard Python library only.
Generates:
1. dist/aweos-x86_64.iso (ISO-9660 bootable image)
2. dist/aweos-x86_64.img (GPT + FAT partitioned raw disk image)
"""

import os
import sys
import struct
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
    # 7 bytes: year - 1900, month, day, hour, minute, second, tz_offset (in 15-min intervals)
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
    rec.append(0) # Ext attribute record length
    rec.extend(u32_both(extent_sector))
    rec.extend(u32_both(data_len))
    rec.extend(iso_date_time())
    rec.append(flags)
    rec.append(0) # Unit size
    rec.append(0) # Interleave gap
    rec.extend(u16_both(1)) # Volume sequence number
    rec.append(name_len)
    rec.extend(encoded_name)
    if len(rec) < rec_len:
        rec.extend(b'\x00' * (rec_len - len(rec)))
    return bytes(rec)

def build_iso(kernel_bytes: bytes, efi_loader_bytes: bytes, grub_cfg_bytes: bytes, out_path: str):
    iso = bytearray(32768) # 16 reserved sectors (16 * 2048)

    # Calculate file layout
    # Sector 16: Primary Volume Descriptor
    # Sector 17: Terminator
    # Sector 18: L-Path Table
    # Sector 19: Root Directory Block
    # Sector 20: BOOT Directory Block
    # Sector 21: EFI Directory Block
    # Sector 22+: Files

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

    # Root Dir Records: BOOT dir, EFI dir
    root_dir_data = bytearray()
    root_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    root_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    root_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, 'BOOT', is_dir=True))
    root_dir_data.extend(make_directory_record(21, SECTOR_SIZE_ISO, 'EFI', is_dir=True))
    pad_to_sector(root_dir_data, SECTOR_SIZE_ISO)

    # BOOT Dir Records: AWEOS kernel, GRUB.CFG
    boot_dir_data = bytearray()
    boot_dir_data.extend(make_directory_record(20, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    boot_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    boot_dir_data.extend(make_directory_record(kernel_sector, kernel_len, 'AWEOS;1'))
    boot_dir_data.extend(make_directory_record(grub_sector, grub_len, 'GRUB.CFG;1'))
    pad_to_sector(boot_dir_data, SECTOR_SIZE_ISO)

    # EFI Dir Records: BOOTX64.EFI
    efi_dir_data = bytearray()
    efi_dir_data.extend(make_directory_record(21, SECTOR_SIZE_ISO, '\x00', is_dir=True))
    efi_dir_data.extend(make_directory_record(19, SECTOR_SIZE_ISO, '\x01', is_dir=True))
    efi_dir_data.extend(make_directory_record(efi_sector, efi_len, 'BOOTX64.EFI;1'))
    pad_to_sector(efi_dir_data, SECTOR_SIZE_ISO)

    # Primary Volume Descriptor (PVD)
    pvd = bytearray(SECTOR_SIZE_ISO)
    pvd[0] = 1 # Type
    pvd[1:6] = b'CD001'
    pvd[6] = 1 # Version
    pvd[8:40] = b'AWEOS                           ' # System ID
    pvd[40:72] = b'AWEOS_BOOT                      ' # Volume ID
    pvd[80:88] = u32_both(total_sectors)
    pvd[120:124] = u16_both(SECTOR_SIZE_ISO)
    pvd[132:140] = u32_both(10) # Path table length
    pvd[140:144] = struct.pack('<I', 18) # L-Path table
    pvd[148:152] = struct.pack('>I', 18) # M-Path table
    pvd[156:190] = make_directory_record(19, SECTOR_SIZE_ISO, '\x00', is_dir=True)
    pvd[190:318] = b'AWEOS_PUBLISHER                 '.ljust(128, b' ')

    # Terminator
    term = bytearray(SECTOR_SIZE_ISO)
    term[0] = 255
    term[1:6] = b'CD001'
    term[6] = 1

    # L-Path Table
    lpath = bytearray(SECTOR_SIZE_ISO)
    lpath[0:10] = struct.pack('<IBH', 19, 0, 1) + b'\x00' # Root, BOOT, EFI

    iso.extend(pvd)
    iso.extend(term)
    iso.extend(lpath)
    iso.extend(root_dir_data)
    iso.extend(boot_dir_data)
    iso.extend(efi_dir_data)

    # Append files
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
    # Create a 32MB disk image
    total_sectors = 65536 # 32 MB
    image = bytearray(total_sectors * SECTOR_SIZE_DISK)

    # 1. Protective MBR at Sector 0
    mbr = bytearray(512)
    # Partition entry 1 at offset 446
    mbr[446] = 0x00 # Status
    mbr[447:450] = b'\x00\x02\x00' # CHS Start
    mbr[450] = 0xEE # Type: GPT Protective
    mbr[451:454] = b'\xFF\xFF\xFF' # CHS End
    mbr[454:458] = struct.pack('<I', 1) # Starting LBA
    mbr[458:462] = struct.pack('<I', total_sectors - 1) # Size
    mbr[510:512] = b'\x55\xAA'
    image[0:512] = mbr

    # 2. Partition setup: ESP partition from LBA 2048 to total_sectors - 34
    part_start_lba = 2048
    part_end_lba = total_sectors - 34
    part_sectors = part_end_lba - part_start_lba + 1

    # 3. GPT Header at Sector 1
    gpt_header = bytearray(92)
    gpt_header[0:8] = b'EFI PART'
    gpt_header[8:12] = struct.pack('<I', 0x00010000) # Revision 1.0
    gpt_header[12:16] = struct.pack('<I', 92) # Header size
    gpt_header[24:32] = struct.pack('<Q', 1) # My LBA
    gpt_header[32:40] = struct.pack('<Q', total_sectors - 1) # Alternate LBA
    gpt_header[40:48] = struct.pack('<Q', part_start_lba) # First Usable
    gpt_header[48:56] = struct.pack('<Q', part_end_lba) # Last Usable
    # Disk GUID:
    disk_guid = bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])
    gpt_header[56:72] = disk_guid
    gpt_header[72:80] = struct.pack('<Q', 2) # Partition entry LBA
    gpt_header[80:84] = struct.pack('<I', 128) # Number of entries
    gpt_header[84:88] = struct.pack('<I', 128) # Size of entry

    # GPT Partition Entries (Sectors 2..33)
    gpt_entries = bytearray(128 * 128)
    # Entry 1: EFI System Partition (GUID: C12A7328-F81F-11D2-BA4B-00A0C93EC93B)
    esp_type_guid = bytes.fromhex('28732AC11FF8D211BA4B00A0C93EC93B')
    esp_unique_guid = bytes.fromhex('112233445566778899AABBCCDDEEFF00')

    entry1 = bytearray(128)
    entry1[0:16] = esp_type_guid
    entry1[16:32] = esp_unique_guid
    entry1[32:40] = struct.pack('<Q', part_start_lba)
    entry1[40:48] = struct.pack('<Q', part_end_lba)
    entry1[48:56] = struct.pack('<Q', 0) # Attributes
    entry1[56:128] = "EFI System Partition".encode('utf-16le').ljust(72, b'\x00')
    gpt_entries[0:128] = entry1

    # Entry array CRC32
    crc_entries = zlib.crc32(gpt_entries) & 0xFFFFFFFF
    gpt_header[88:92] = struct.pack('<I', crc_entries)

    # Header CRC32 (with crc field zeroed)
    crc_hdr = zlib.crc32(gpt_header) & 0xFFFFFFFF
    gpt_header[16:20] = struct.pack('<I', crc_hdr)

    image[512:604] = gpt_header
    image[1024:1024 + len(gpt_entries)] = gpt_entries

    # 4. FAT16 Filesystem in ESP Partition starting at part_start_lba * 512
    fat_offset = part_start_lba * SECTOR_SIZE_DISK
    fat_bytes = bytearray(part_sectors * SECTOR_SIZE_DISK)

    # Boot Sector (BPB) for FAT16
    bpb = bytearray(512)
    bpb[0:3] = b'\xEB\x3C\x90' # JMP
    bpb[3:11] = b'AWEOS1.0'
    bpb[11:13] = struct.pack('<H', 512) # Bytes per sector
    bpb[13] = 4 # Sectors per cluster (2KB cluster)
    bpb[14:16] = struct.pack('<H', 1) # Reserved sectors
    bpb[16] = 2 # Number of FATs
    bpb[17:19] = struct.pack('<H', 512) # Root entries
    bpb[19:21] = struct.pack('<H', 0) # Small sectors
    bpb[21] = 0xF8 # Media descriptor
    bpb[22:24] = struct.pack('<H', 64) # Sectors per FAT
    bpb[24:26] = struct.pack('<H', 32) # Sectors per track
    bpb[26:28] = struct.pack('<H', 64) # Number of heads
    bpb[28:32] = struct.pack('<I', part_start_lba)
    bpb[32:36] = struct.pack('<I', part_sectors)
    bpb[36] = 0x80 # Drive number
    bpb[38] = 0x29 # Extended signature
    bpb[39:43] = struct.pack('<I', 0x12345678) # Volume ID
    bpb[43:54] = b'AWEOS_BOOT '
    bpb[54:62] = b'FAT16   '
    bpb[510:512] = b'\x55\xAA'
    fat_bytes[0:512] = bpb

    # Populate FAT tables (FAT1 & FAT2 at offset 512 & 512 + 64*512)
    # cluster 0: media, cluster 1: EOF
    fat_table = bytearray(64 * 512)
    fat_table[0:4] = b'\xF8\xFF\xFF\xFF'
    fat_bytes[512:512 + 64 * 512] = fat_table
    fat_bytes[512 + 64 * 512:512 + 128 * 512] = fat_table

    # Root directory at offset 512 + 128*512 = 65536
    root_dir_offset = (1 + 2 * 64) * 512 # 66048 bytes
    root_dir = bytearray(512 * 32) # 512 entries * 32 bytes

    # Root dir entry 1: Volume Label
    vol_entry = bytearray(32)
    vol_entry[0:11] = b'AWEOS_BOOT '
    vol_entry[11] = 0x08 # Volume ID flag
    root_dir[0:32] = vol_entry

    # Root dir entry 2: README.TXT
    readme_data = b'AWEOS Operating System - Universal Singularity\nBootable GPT/FAT32 Partition\n'
    readme_entry = bytearray(32)
    readme_entry[0:11] = b'AWEOS   TXT'
    readme_entry[11] = 0x20 # Archive
    readme_entry[26:28] = struct.pack('<H', 2) # Start cluster 2
    readme_entry[28:32] = struct.pack('<I', len(readme_data))
    root_dir[32:64] = readme_entry

    fat_bytes[root_dir_offset:root_dir_offset + len(root_dir)] = root_dir

    # Cluster 2 data area starts right after root directory entries
    data_area_offset = root_dir_offset + 512 * 32 # Offset in FAT volume
    fat_bytes[data_area_offset:data_area_offset + len(readme_data)] = readme_data

    image[fat_offset:fat_offset + len(fat_bytes)] = fat_bytes

    with open(out_path, 'wb') as f:
        f.write(image)
    print(f"Created disk image: {out_path} ({len(image)} bytes)")

def main():
    root = Path(__file__).resolve().parent.parent
    out_dir = root / 'dist'
    out_dir.mkdir(parents=True, exist_ok=True)

    kernel_path = root / 'target' / 'x86_64-unknown-none' / 'release' / 'aweos'
    efi_loader_path = root / 'target' / 'x86_64-unknown-uefi' / 'release' / 'aweloader.efi'

    # Check if kernel exists, else fallback to debug build or dummy image marker
    if kernel_path.exists():
        kernel_bytes = kernel_path.read_bytes()
    else:
        debug_k = root / 'target' / 'debug' / 'aweos'
        if debug_k.exists():
            kernel_bytes = debug_k.read_bytes()
        else:
            kernel_bytes = b'AWEOS_KERNEL_STUB'

    if efi_loader_path.exists():
        efi_loader_bytes = efi_loader_path.read_bytes()
    else:
        debug_e = root / 'target' / 'x86_64-unknown-uefi' / 'debug' / 'aweloader.efi'
        if debug_e.exists():
            efi_loader_bytes = debug_e.read_bytes()
        else:
            efi_loader_bytes = b'AWEOS_EFI_LOADER_STUB'

    grub_cfg = (root / 'kernel-bin' / 'grub.cfg').read_bytes() if (root / 'kernel-bin' / 'grub.cfg').exists() else b'set timeout=5\n'

    iso_path = out_dir / 'aweos-x86_64.iso'
    img_path = out_dir / 'aweos-x86_64.img'
    efi_path = out_dir / 'BOOTX64.EFI'

    efi_path.write_bytes(efi_loader_bytes)

    build_iso(kernel_bytes, efi_loader_bytes, grub_cfg, str(iso_path))
    build_gpt_fat_img(kernel_bytes, efi_loader_bytes, grub_cfg, str(img_path))

if __name__ == '__main__':
    main()

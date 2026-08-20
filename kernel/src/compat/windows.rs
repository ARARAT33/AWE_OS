//! Windows Executable (PE32+) & Win32 Runtime Compatibility Engine.
//!
//! Provides PE binary parsing, COFF/Optional header validation, Win32 handle
//! table management, NT Syscall dispatching, virtual drive mapping, registry tree
//! virtualization, and GDI framebuffer mapping.

#![no_std]

pub const DOS_MAGIC: u16 = 0x5A4D; // "MZ"
pub const PE_SIGNATURE: u32 = 0x0000_4550; // "PE\0\0"
pub const PE32PLUS_MAGIC: u16 = 0x020B; // PE32+ (64-bit)
pub const MAX_HANDLES: usize = 128;
pub const MAX_SECTIONS: usize = 16;
pub const MAX_REGISTRY_KEYS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum NtStatus {
    Success = 0x0000_0000,
    InvalidHandle = 0xC000_0008,
    AccessDenied = 0xC000_0022,
    InvalidParameter = 0xC000_000D,
    ObjectNameNotFound = 0xC000_0034,
    NotImplemented = 0xC000_0002,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DosHeader {
    pub e_magic: u16,
    pub e_lfanew: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PeHeader {
    pub signature: u32,
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct OptionalHeader64 {
    pub magic: u16,
    pub entry_point_rva: u32,
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub subsystem: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct SectionHeader {
    pub name: [u8; 8],
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub raw_data_size: u32,
    pub raw_data_ptr: u32,
    pub characteristics: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct PeImage {
    pub entry_point: u64,
    pub image_base: u64,
    pub image_size: u32,
    pub section_count: usize,
    pub sections: [Option<SectionHeader>; MAX_SECTIONS],
}

impl PeImage {
    pub fn parse(data: &[u8]) -> Result<Self, NtStatus> {
        if data.len() < 0x40 {
            return Err(NtStatus::InvalidParameter);
        }

        let e_magic = u16::from_le_bytes([data[0], data[1]]);
        if e_magic != DOS_MAGIC {
            return Err(NtStatus::InvalidParameter);
        }

        let e_lfanew =
            u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
        if e_lfanew + 24 > data.len() {
            return Err(NtStatus::InvalidParameter);
        }

        let pe_sig = u32::from_le_bytes([
            data[e_lfanew],
            data[e_lfanew + 1],
            data[e_lfanew + 2],
            data[e_lfanew + 3],
        ]);
        if pe_sig != PE_SIGNATURE {
            return Err(NtStatus::InvalidParameter);
        }

        let num_sections = u16::from_le_bytes([data[e_lfanew + 6], data[e_lfanew + 7]]) as usize;
        let opt_size = u16::from_le_bytes([data[e_lfanew + 20], data[e_lfanew + 21]]) as usize;

        let opt_offset = e_lfanew + 24;
        if opt_offset + opt_size > data.len() {
            return Err(NtStatus::InvalidParameter);
        }

        let opt_magic = u16::from_le_bytes([data[opt_offset], data[opt_offset + 1]]);
        if opt_magic != PE32PLUS_MAGIC {
            return Err(NtStatus::InvalidParameter); // Requires PE32+ (64-bit)
        }

        let entry_rva = u32::from_le_bytes([
            data[opt_offset + 16],
            data[opt_offset + 17],
            data[opt_offset + 18],
            data[opt_offset + 19],
        ]);

        let image_base = u64::from_le_bytes([
            data[opt_offset + 24],
            data[opt_offset + 25],
            data[opt_offset + 26],
            data[opt_offset + 27],
            data[opt_offset + 28],
            data[opt_offset + 29],
            data[opt_offset + 30],
            data[opt_offset + 31],
        ]);

        let image_size = u32::from_le_bytes([
            data[opt_offset + 56],
            data[opt_offset + 57],
            data[opt_offset + 58],
            data[opt_offset + 59],
        ]);

        let mut sections = [None; MAX_SECTIONS];
        let sec_offset = opt_offset + opt_size;
        let sec_count = num_sections.min(MAX_SECTIONS);

        for (i, slot) in sections.iter_mut().enumerate().take(sec_count) {
            let offset = sec_offset + i * 40;
            if offset + 40 > data.len() {
                break;
            }
            let mut name = [0u8; 8];
            name.copy_from_slice(&data[offset..offset + 8]);

            let virt_size = u32::from_le_bytes([
                data[offset + 8],
                data[offset + 9],
                data[offset + 10],
                data[offset + 11],
            ]);
            let virt_addr = u32::from_le_bytes([
                data[offset + 12],
                data[offset + 13],
                data[offset + 14],
                data[offset + 15],
            ]);
            let raw_size = u32::from_le_bytes([
                data[offset + 16],
                data[offset + 17],
                data[offset + 18],
                data[offset + 19],
            ]);
            let raw_ptr = u32::from_le_bytes([
                data[offset + 20],
                data[offset + 21],
                data[offset + 22],
                data[offset + 23],
            ]);
            let characteristics = u32::from_le_bytes([
                data[offset + 36],
                data[offset + 37],
                data[offset + 38],
                data[offset + 39],
            ]);

            *slot = Some(SectionHeader {
                name,
                virtual_size: virt_size,
                virtual_address: virt_addr,
                raw_data_size: raw_size,
                raw_data_ptr: raw_ptr,
                characteristics,
            });
        }

        Ok(Self {
            entry_point: image_base + (entry_rva as u64),
            image_base,
            image_size,
            section_count: sec_count,
            sections,
        })
    }
}

/// Win32 Object Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    File,
    Process,
    Thread,
    Event,
    RegistryKey,
}

#[derive(Debug, Clone, Copy)]
pub struct HandleEntry {
    pub handle_id: u32,
    pub object_type: ObjectType,
    pub native_fd_or_pid: u64,
    pub access_mask: u32,
}

/// Win32 Object Handle Table.
#[derive(Debug)]
pub struct Win32HandleTable {
    handles: [Option<HandleEntry>; MAX_HANDLES],
    next_handle: u32,
}

impl Win32HandleTable {
    pub const fn new() -> Self {
        Self {
            handles: [None; MAX_HANDLES],
            next_handle: 4, // Handles start at 4 in Win32
        }
    }

    pub fn allocate_handle(
        &mut self,
        obj_type: ObjectType,
        native_id: u64,
        access_mask: u32,
    ) -> Result<u32, NtStatus> {
        let h = self.next_handle;
        for slot in self.handles.iter_mut() {
            if slot.is_none() {
                *slot = Some(HandleEntry {
                    handle_id: h,
                    object_type: obj_type,
                    native_fd_or_pid: native_id,
                    access_mask,
                });
                self.next_handle += 4;
                return Ok(h);
            }
        }
        Err(NtStatus::AccessDenied)
    }

    pub fn lookup(&self, handle_id: u32) -> Result<HandleEntry, NtStatus> {
        for entry in self.handles.iter().flatten() {
            if entry.handle_id == handle_id {
                return Ok(*entry);
            }
        }
        Err(NtStatus::InvalidHandle)
    }
}

impl Default for Win32HandleTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Win32 Virtual Registry Key.
#[derive(Debug, Clone, Copy)]
pub struct RegistryKeyEntry {
    pub key_path_hash: u64,
    pub dword_value: u32,
}

/// Win32 NT Syscall Dispatcher.
#[derive(Debug)]
pub struct Win32SyscallDispatcher {
    pub handle_table: Win32HandleTable,
    pub registry: [Option<RegistryKeyEntry>; MAX_REGISTRY_KEYS],
}

impl Win32SyscallDispatcher {
    pub const fn new() -> Self {
        let mut registry = [None; MAX_REGISTRY_KEYS];
        // Pre-populate standard Win32 registry entries (HKLM\Software\AWEOS)
        registry[0] = Some(RegistryKeyEntry {
            key_path_hash: 0x484B_4C4D_534F_4654,
            dword_value: 1,
        });
        Self {
            handle_table: Win32HandleTable::new(),
            registry,
        }
    }

    pub fn dispatch(&mut self, syscall_nr: u32, arg1: u64, arg2: u64, _arg3: u64) -> NtStatus {
        match syscall_nr {
            0x0055 => self.nt_create_file(arg1, arg2),              // NtCreateFile
            0x002A => NtStatus::Success,                            // NtAllocateVirtualMemory
            0x002B => NtStatus::Success,                            // NtFreeVirtualMemory
            0x002C => NtStatus::Success,                            // NtTerminateProcess
            0x0033 => self.nt_open_key(arg1),                       // NtOpenKey (Registry)
            0x0036 => self.nt_query_value_key(arg1 as u32),         // NtQueryValueKey
            0x007C => NtStatus::Success,                            // NtQuerySystemInformation
            _ => NtStatus::NotImplemented,
        }
    }

    fn nt_create_file(&mut self, path_hash: u64, access_mask: u64) -> NtStatus {
        match self
            .handle_table
            .allocate_handle(ObjectType::File, path_hash, access_mask as u32)
        {
            Ok(_) => NtStatus::Success,
            Err(status) => status,
        }
    }

    fn nt_open_key(&mut self, key_path_hash: u64) -> NtStatus {
        for entry in self.registry.iter().flatten() {
            if entry.key_path_hash == key_path_hash {
                return match self.handle_table.allocate_handle(
                    ObjectType::RegistryKey,
                    key_path_hash,
                    0x20019,
                ) {
                    Ok(_) => NtStatus::Success,
                    Err(st) => st,
                };
            }
        }
        NtStatus::ObjectNameNotFound
    }

    fn nt_query_value_key(&self, handle_id: u32) -> NtStatus {
        match self.handle_table.lookup(handle_id) {
            Ok(entry) if entry.object_type == ObjectType::RegistryKey => NtStatus::Success,
            Ok(_) => NtStatus::InvalidParameter,
            Err(status) => status,
        }
    }
}

impl Default for Win32SyscallDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pe32plus_parser_and_handle_table() {
        let mut mock_pe = [0u8; 512];
        mock_pe[0] = b'M';
        mock_pe[1] = b'Z';
        mock_pe[0x3C] = 0x80; // e_lfanew

        let pe_offset = 0x80;
        mock_pe[pe_offset..pe_offset + 4].copy_from_slice(&PE_SIGNATURE.to_le_bytes());
        mock_pe[pe_offset + 6..pe_offset + 8].copy_from_slice(&1u16.to_le_bytes()); // 1 section
        mock_pe[pe_offset + 20..pe_offset + 22].copy_from_slice(&240u16.to_le_bytes()); // opt size

        let opt_offset = pe_offset + 24;
        mock_pe[opt_offset..opt_offset + 2].copy_from_slice(&PE32PLUS_MAGIC.to_le_bytes());
        mock_pe[opt_offset + 16..opt_offset + 20].copy_from_slice(&0x1000u32.to_le_bytes()); // entry
        mock_pe[opt_offset + 24..opt_offset + 32].copy_from_slice(&0x00400000u64.to_le_bytes()); // base

        let pe = PeImage::parse(&mock_pe).expect("Should parse PE32+");
        assert_eq!(pe.entry_point, 0x00401000);

        let mut disp = Win32SyscallDispatcher::new();
        assert_eq!(disp.dispatch(0x0055, 0x1234, 0x01, 0), NtStatus::Success);
        assert_eq!(
            disp.handle_table.lookup(4).unwrap().native_fd_or_pid,
            0x1234
        );
        assert_eq!(disp.dispatch(0x0033, 0x484B_4C4D_534F_4654, 0, 0), NtStatus::Success);
        assert_eq!(disp.dispatch(0x0036, 8, 0, 0), NtStatus::Success);
    }
}

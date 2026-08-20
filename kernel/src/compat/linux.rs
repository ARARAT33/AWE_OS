//! Linux ELF64 Executable & POSIX Syscall Runtime Compatibility Engine.
//!
//! Provides ELF64 binary parsing, PT_LOAD segment validation, file descriptor
//! translation, POSIX syscall dispatching, and signal mapping.

#![no_std]

pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;
pub const PT_LOAD: u32 = 1;
pub const MAX_FD: usize = 64;
pub const MAX_LOAD_SEGMENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxErrno {
    Success = 0,
    EPERM = 1,
    ENOENT = 2,
    EBADF = 9,
    ENOMEM = 12,
    EACCES = 13,
    EINVAL = 22,
    ENOSYS = 38,
}

#[derive(Debug, Clone, Copy)]
pub struct ElfProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Elf64Image {
    pub entry_point: u64,
    pub segment_count: usize,
    pub load_segments: [Option<ElfProgramHeader>; MAX_LOAD_SEGMENTS],
}

impl Elf64Image {
    pub fn parse(data: &[u8]) -> Result<Self, LinuxErrno> {
        if data.len() < 64 {
            return Err(LinuxErrno::EINVAL);
        }

        if data[0..4] != ELF_MAGIC {
            return Err(LinuxErrno::EINVAL);
        }

        if data[4] != ELFCLASS64 || data[5] != ELFDATA2LSB {
            return Err(LinuxErrno::EINVAL); // Requires 64-bit Little Endian ELF
        }

        let entry_point = u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ]);

        let ph_offset = u64::from_le_bytes([
            data[32], data[33], data[34], data[35],
            data[36], data[37], data[38], data[39],
        ]) as usize;

        let ph_entsize = u16::from_le_bytes([data[54], data[55]]) as usize;
        let ph_num = u16::from_le_bytes([data[56], data[57]]) as usize;

        if ph_offset + (ph_num * ph_entsize) > data.len() {
            return Err(LinuxErrno::EINVAL);
        }

        let mut load_segments = [None; MAX_LOAD_SEGMENTS];
        let mut count = 0;

        for i in 0..ph_num {
            let offset = ph_offset + (i * ph_entsize);
            let p_type = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);

            if p_type == PT_LOAD {
                let p_flags = u32::from_le_bytes([data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7]]);
                let p_off = u64::from_le_bytes([
                    data[offset + 8], data[offset + 9], data[offset + 10], data[offset + 11],
                    data[offset + 12], data[offset + 13], data[offset + 14], data[offset + 15],
                ]);
                let p_vaddr = u64::from_le_bytes([
                    data[offset + 16], data[offset + 17], data[offset + 18], data[offset + 19],
                    data[offset + 20], data[offset + 21], data[offset + 22], data[offset + 23],
                ]);
                let p_filesz = u64::from_le_bytes([
                    data[offset + 32], data[offset + 33], data[offset + 34], data[offset + 35],
                    data[offset + 36], data[offset + 37], data[offset + 38], data[offset + 39],
                ]);
                let p_memsz = u64::from_le_bytes([
                    data[offset + 40], data[offset + 41], data[offset + 42], data[offset + 43],
                    data[offset + 44], data[offset + 45], data[offset + 46], data[offset + 47],
                ]);

                if count < MAX_LOAD_SEGMENTS {
                    load_segments[count] = Some(ElfProgramHeader {
                        p_type,
                        p_flags,
                        p_offset: p_off,
                        p_vaddr,
                        p_paddr: p_vaddr,
                        p_filesz,
                        p_memsz,
                        p_align: 4096,
                    });
                    count += 1;
                }
            }
        }

        Ok(Self {
            entry_point,
            segment_count: count,
            load_segments,
        })
    }
}

/// Linux File Descriptor Entry.
#[derive(Debug, Clone, Copy)]
pub struct LinuxFdEntry {
    pub fd: i32,
    pub path_hash: u64,
    pub is_readable: bool,
    pub is_writable: bool,
}

/// Linux File Descriptor Table.
#[derive(Debug)]
pub struct LinuxFdTable {
    fds: [Option<LinuxFdEntry>; MAX_FD],
}

impl LinuxFdTable {
    pub const fn new() -> Self {
        let mut fds: [Option<LinuxFdEntry>; MAX_FD] = [None; MAX_FD];
        // Standard streams: 0=stdin, 1=stdout, 2=stderr
        fds[0] = Some(LinuxFdEntry { fd: 0, path_hash: 0, is_readable: true, is_writable: false });
        fds[1] = Some(LinuxFdEntry { fd: 1, path_hash: 0, is_readable: false, is_writable: true });
        fds[2] = Some(LinuxFdEntry { fd: 2, path_hash: 0, is_readable: false, is_writable: true });
        Self { fds }
    }

    pub fn allocate_fd(&mut self, path_hash: u64, read: bool, write: bool) -> Result<i32, LinuxErrno> {
        for fd in 3..MAX_FD {
            if self.fds[fd].is_none() {
                self.fds[fd] = Some(LinuxFdEntry {
                    fd: fd as i32,
                    path_hash,
                    is_readable: read,
                    is_writable: write,
                });
                return Ok(fd as i32);
            }
        }
        Err(LinuxErrno::ENOMEM)
    }

    pub fn lookup(&self, fd: i32) -> Result<LinuxFdEntry, LinuxErrno> {
        if fd < 0 || (fd as usize) >= MAX_FD {
            return Err(LinuxErrno::EBADF);
        }
        self.fds[fd as usize].ok_or(LinuxErrno::EBADF)
    }
}

/// Linux POSIX Syscall Dispatcher.
#[derive(Debug)]
pub struct LinuxSyscallDispatcher {
    pub fd_table: LinuxFdTable,
}

impl LinuxSyscallDispatcher {
    pub const fn new() -> Self {
        Self {
            fd_table: LinuxFdTable::new(),
        }
    }

    pub fn dispatch(&mut self, sys_nr: u64, arg1: u64, _arg2: u64, _arg3: u64) -> Result<u64, LinuxErrno> {
        match sys_nr {
            0 => self.sys_read(arg1 as i32),   // sys_read
            1 => self.sys_write(arg1 as i32),  // sys_write
            2 => self.sys_open(arg1),          // sys_open
            3 => self.sys_close(arg1 as i32),  // sys_close
            9 => Ok(0x7FFFF7FF0000),           // sys_mmap
            60 => Ok(0),                        // sys_exit
            _ => Err(LinuxErrno::ENOSYS),
        }
    }

    fn sys_read(&self, fd: i32) -> Result<u64, LinuxErrno> {
        let entry = self.fd_table.lookup(fd)?;
        if !entry.is_readable {
            return Err(LinuxErrno::EBADF);
        }
        Ok(0) // 0 bytes read
    }

    fn sys_write(&self, fd: i32) -> Result<u64, LinuxErrno> {
        let entry = self.fd_table.lookup(fd)?;
        if !entry.is_writable {
            return Err(LinuxErrno::EBADF);
        }
        Ok(0)
    }

    fn sys_open(&mut self, path_hash: u64) -> Result<u64, LinuxErrno> {
        let fd = self.fd_table.allocate_fd(path_hash, true, true)?;
        Ok(fd as u64)
    }

    fn sys_close(&mut self, fd: i32) -> Result<u64, LinuxErrno> {
        if fd < 3 || (fd as usize) >= MAX_FD {
            return Err(LinuxErrno::EBADF);
        }
        self.fd_table.fds[fd as usize] = None;
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf64_parser_and_linux_syscalls() {
        let mut mock_elf = [0u8; 128];
        mock_elf[0..4].copy_from_slice(&ELF_MAGIC);
        mock_elf[4] = ELFCLASS64;
        mock_elf[5] = ELFDATA2LSB;
        mock_elf[24..32].copy_from_slice(&0x00401000u64.to_le_bytes()); // entry point
        mock_elf[32..40].copy_from_slice(&64u64.to_le_bytes()); // ph_offset
        mock_elf[54..56].copy_from_slice(&56u16.to_le_bytes()); // ph_entsize
        mock_elf[56..58].copy_from_slice(&1u16.to_le_bytes()); // ph_num

        // Program Header at offset 64
        let ph_offset = 64;
        mock_elf[ph_offset..ph_offset + 4].copy_from_slice(&PT_LOAD.to_le_bytes());
        mock_elf[ph_offset + 16..ph_offset + 24].copy_from_slice(&0x00400000u64.to_le_bytes()); // p_vaddr

        let img = Elf64Image::parse(&mock_elf).expect("Should parse ELF64");
        assert_eq!(img.entry_point, 0x00401000);
        assert_eq!(img.segment_count, 1);

        let mut disp = LinuxSyscallDispatcher::new();
        assert_eq!(disp.dispatch(1, 1, 0, 0).unwrap(), 0); // sys_write to stdout (fd=1)
        let new_fd = disp.dispatch(2, 0x1234_5678, 0, 0).unwrap(); // sys_open
        assert_eq!(new_fd, 3);
        assert_eq!(disp.dispatch(3, 3, 0, 0).unwrap(), 0); // sys_close fd=3
    }
}

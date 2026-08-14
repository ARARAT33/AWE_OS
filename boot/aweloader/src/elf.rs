#![no_std]

pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
pub const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Header {
    pub ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub struct LoadSegment {
    pub file_offset: u64,
    pub virtual_address: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub flags: u32,
}

pub fn parse<'a>(image: &'a [u8]) -> Option<(&'a Elf64Header, &'a [u8])> {
    if image.len() < core::mem::size_of::<Elf64Header>() { return None; }
    let h = unsafe { &*(image.as_ptr() as *const Elf64Header) };
    if h.ident[..4] != ELF_MAGIC || h.ident[4] != 2 || h.ident[5] != 1 { return None; }
    if h.e_phentsize as usize != core::mem::size_of::<Elf64ProgramHeader>() { return None; }
    let ph_size = (h.e_phentsize as usize).checked_mul(h.e_phnum as usize)?;
    let ph_end = (h.e_phoff as usize).checked_add(ph_size)?;
    if ph_end > image.len() { return None; }
    Some((h, &image[h.e_phoff as usize..ph_end]))
}

pub fn segment(image: &[u8], ph: &Elf64ProgramHeader) -> Option<LoadSegment> {
    if ph.p_type != PT_LOAD { return None; }
    let end = ph.p_offset.checked_add(ph.p_filesz)?;
    if end > image.len() as u64 || ph.p_memsz < ph.p_filesz { return None; }
    Some(LoadSegment { file_offset: ph.p_offset, virtual_address: ph.p_vaddr, file_size: ph.p_filesz, memory_size: ph.p_memsz, flags: ph.p_flags })
}

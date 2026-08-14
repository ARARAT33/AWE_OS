#![no_std]

pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
pub const PT_LOAD: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64Header {
    pub ident: [u8; 16],
    pub typ: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub phoff: u64,
    pub shoff: u64,
    pub flags: u32,
    pub ehsize: u16,
    pub phentsize: u16,
    pub phnum: u16,
    pub shentsize: u16,
    pub shnum: u16,
    pub shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProgramHeader64 {
    pub typ: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

pub fn parse_header(image: &[u8]) -> Option<&Elf64Header> {
    if image.len() < core::mem::size_of::<Elf64Header>() { return None; }
    let h = unsafe { &*(image.as_ptr() as *const Elf64Header) };
    if h.ident[..4] != ELF_MAGIC || h.ident[4] != 2 || h.ident[5] != 1 { return None; }
    if h.phentsize as usize != core::mem::size_of::<ProgramHeader64>() { return None; }
    let ph_end = h.phoff.checked_add((h.phentsize as u64).checked_mul(h.phnum as u64)?)?;
    if ph_end > image.len() as u64 { return None; }
    Some(h)
}

pub fn loadable_segments<'a>(image: &'a [u8], h: &Elf64Header) -> Option<impl Iterator<Item = &'a ProgramHeader64>> {
    let base = image.as_ptr() as usize;
    let len = image.len();
    let count = h.phnum as usize;
    let stride = h.phentsize as usize;
    let offset = h.phoff as usize;
    if offset.checked_add(count.checked_mul(stride)?)? > len { return None; }
    Some((0..count).filter_map(move |i| {
        let p = base.checked_add(offset)?.checked_add(i.checked_mul(stride)?)?;
        let ph = unsafe { &*(p as *const ProgramHeader64) };
        (ph.typ == PT_LOAD).then_some(ph)
    }))
}

#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Frame {
    pub start: u64,
}

pub const PAGE_SIZE: u64 = 4096;

pub const fn frame_at(address: u64) -> Frame {
    Frame { start: address & !(PAGE_SIZE - 1) }
}

pub const fn frame_end(frame: Frame) -> u64 {
    frame.start + PAGE_SIZE
}

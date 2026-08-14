//! Minimal IPC contracts. The kernel exposes handles/capabilities rather than raw object pointers.

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Handle(pub u32);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Capability(pub u64);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MessageHeader {
    pub channel: Handle,
    pub length: u32,
    pub flags: u32,
    pub sequence: u64,
}

pub const MAX_INLINE: usize = 256;

#[repr(C)]
pub struct Message {
    pub header: MessageHeader,
    pub data: [u8; MAX_INLINE],
}

impl Message {
    pub const fn empty(channel: Handle, sequence: u64) -> Self {
        Self {
            header: MessageHeader { channel, length: 0, flags: 0, sequence },
            data: [0; MAX_INLINE],
        }
    }
}

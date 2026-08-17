#![no_std]

#[repr(C)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

pub type Handler = extern "C" fn(&mut InterruptFrame);

pub struct InterruptTable {
    handlers: [Option<Handler>; 256],
}

impl Default for InterruptTable {
    fn default() -> Self {
        Self::new()
    }
}

impl InterruptTable {
    pub const fn new() -> Self {
        Self {
            handlers: [None; 256],
        }
    }

    pub fn register(&mut self, vector: u8, handler: Handler) {
        self.handlers[vector as usize] = Some(handler);
    }

    pub fn get(&self, vector: u8) -> Option<Handler> {
        self.handlers[vector as usize]
    }
}

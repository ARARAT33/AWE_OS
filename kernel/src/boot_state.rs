#![no_std]

use awe_boot_protocol::{validate, BootInfo};

pub struct EarlyBoot<'a> {
    info: &'a BootInfo,
}

impl<'a> EarlyBoot<'a> {
    pub fn new(info: &'a BootInfo) -> Option<Self> {
        if validate(info) { Some(Self { info }) } else { None }
    }

    pub const fn info(&self) -> &BootInfo { self.info }
}

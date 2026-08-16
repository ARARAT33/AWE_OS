#![no_std]

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MmioRegion { pub base: u64, pub length: u64 }
impl MmioRegion {
    pub const fn end(&self) -> Option<u64> { self.base.checked_add(self.length) }
    pub const fn contains(&self, address: u64) -> bool { match self.end() { Some(end) => address >= self.base && address < end, None => false } }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterruptMode { None = 0, Legacy = 1, Msi = 2, MsiX = 3 }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DmaPolicy { pub max_bytes: u64, pub address_bits: u8, pub coherent: bool }

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeviceContract<const M: usize> {
    pub vendor: u16, pub device: u16, pub class_code: u32,
    pub mmio: [Option<MmioRegion>; M], pub interrupt: InterruptMode, pub dma: DmaPolicy,
}
impl<const M: usize> DeviceContract<M> {
    pub const fn valid(&self) -> bool { self.vendor != 0 && self.vendor != 0xffff && self.device != 0 && self.device != 0xffff && self.dma.max_bytes != 0 && self.dma.address_bits >= 32 }
    pub const fn allows_mmio(&self, address: u64) -> bool { let mut i=0; while i<M { if let Some(region)=self.mmio[i] { if region.contains(address) { return true; } } i+=1; } false }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn mmio_range_is_overflow_safe() { let r=MmioRegion{base:0x1000,length:0x100}; assert!(r.contains(0x1080)); assert!(!r.contains(0x1100)); assert!(!MmioRegion{base:u64::MAX,length:2}.contains(u64::MAX)); }
    #[test] fn invalid_device_ids_are_rejected() { let c=DeviceContract::<1>{vendor:0xffff,device:1,class_code:0,mmio:[None],interrupt:InterruptMode::None,dma:DmaPolicy{max_bytes:4096,address_bits:64,coherent:true}}; assert!(!c.valid()); }
}

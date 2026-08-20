#![no_std]
use super::core::{CoreError, DriverSlot, HardwareInfo, LinuxDriverAdapter};
use super::universal::{DriverAbi, DriverOs};
pub struct LinuxLayer<A> {
    pub slot: DriverSlot<A>,
}
impl<A: LinuxDriverAdapter> LinuxLayer<A> {
    pub const fn new(adapter: A) -> Self {
        Self {
            slot: DriverSlot::new(adapter),
        }
    }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Linux
            || (id.abi != DriverAbi::LinuxKmod && id.abi != DriverAbi::LinuxUserMode)
        {
            return Err(CoreError::UnsupportedAbi);
        }
        if !id.signed {
            return Err(CoreError::PolicyDenied);
        }
        if !id.matches(hw) {
            return Err(CoreError::InvalidDevice);
        }
        let api = self.slot.adapter.linux_api_version();
        if api == 0
            || api > u32::from(u16::MAX)
            || id.api_version != api as u16
            || self.slot.adapter.linux_module_name().is_empty()
        {
            return Err(CoreError::InvalidRequest);
        }
        if !hw.valid() || hw.irq == u32::MAX {
            return Err(CoreError::InvalidDevice);
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::super::bus::DeviceId;
    use super::super::core::DriverAdapter;
    use super::*;
    struct Mock {
        identity_api: u32,
        runtime_api: u32,
        name: &'static str,
        vendor: u16,
        device: u16,
        signed: bool,
    }
    impl DriverAdapter for Mock {
        fn identity(&self) -> super::super::core::DriverIdentity {
            super::super::core::DriverIdentity {
                os: DriverOs::Linux,
                abi: DriverAbi::LinuxKmod,
                api_version: self.identity_api as u16,
                vendor: self.vendor,
                device: self.device,
                signed: self.signed,
            }
        }
        fn probe(&mut self, _: &HardwareInfo) -> Result<(), CoreError> {
            Ok(())
        }
        fn start(&mut self, _: &HardwareInfo) -> Result<(), CoreError> {
            Ok(())
        }
        fn stop(&mut self, _: &HardwareInfo) -> Result<(), CoreError> {
            Ok(())
        }
        fn remove(&mut self, _: &HardwareInfo) -> Result<(), CoreError> {
            Ok(())
        }
    }
    impl LinuxDriverAdapter for Mock {
        fn linux_api_version(&self) -> u32 {
            self.runtime_api
        }
        fn linux_module_name(&self) -> &'static str {
            self.name
        }
    }
    fn hw() -> HardwareInfo {
        HardwareInfo {
            id: DeviceId {
                vendor: 1,
                device: 2,
                class: 0,
                revision: 1,
            },
            mmio_base: 0x1000,
            mmio_length: 0x100,
            irq: 5,
            dma_bits: 64,
        }
    }
    fn good() -> LinuxLayer<Mock> {
        LinuxLayer::new(Mock {
            identity_api: 6,
            runtime_api: 6,
            name: "awe-net",
            vendor: 1,
            device: 2,
            signed: true,
        })
    }
    #[test]
    fn valid_adapter_passes() {
        assert!(good().validate(&hw()).is_ok())
    }
    #[test]
    fn api_mismatch_is_rejected() {
        let mut x = good();
        x.slot.adapter.runtime_api = 7;
        assert_eq!(x.validate(&hw()), Err(CoreError::InvalidRequest))
    }
    #[test]
    fn invalid_irq_is_rejected() {
        let x = good();
        let mut h = hw();
        h.irq = u32::MAX;
        assert_eq!(x.validate(&h), Err(CoreError::InvalidDevice))
    }
    #[test]
    fn unsigned_driver_is_rejected() {
        let mut x = good();
        x.slot.adapter.signed = false;
        assert_eq!(x.validate(&hw()), Err(CoreError::PolicyDenied))
    }
}

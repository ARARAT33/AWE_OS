#![no_std]

use super::bus::DeviceId;
use super::universal::{DriverAbi, DriverAction, DriverOs};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CoreError {
    InvalidDevice,
    UnsupportedAbi,
    InvalidRequest,
    PolicyDenied,
    ProbeFailed,
    StartFailed,
    NotBound,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HardwareInfo {
    pub id: DeviceId,
    pub mmio_base: u64,
    pub mmio_length: u64,
    pub irq: u32,
    pub dma_bits: u8,
}

impl HardwareInfo {
    pub const fn valid(&self) -> bool {
        self.id.vendor != 0 && self.id.device != 0 && self.mmio_length != 0 && self.dma_bits >= 32
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DriverIdentity {
    pub os: DriverOs,
    pub abi: DriverAbi,
    pub api_version: u16,
    pub vendor: u16,
    pub device: u16,
    pub signed: bool,
}

impl DriverIdentity {
    pub const fn matches(&self, hw: &HardwareInfo) -> bool {
        self.vendor == hw.id.vendor && self.device == hw.id.device && self.signed
    }
}

/// The only hardware-facing interface exposed to compatibility adapters.
/// A Linux/Windows/Android adapter must translate its native operations into
/// these primitives; it never receives unrestricted CellKernel access.
pub trait HardwareAbstraction {
    fn mmio_read32(&self, hw: &HardwareInfo, offset: u64) -> Result<u32, CoreError>;
    fn mmio_write32(&mut self, hw: &HardwareInfo, offset: u64, value: u32) -> Result<(), CoreError>;
    fn irq_ack(&mut self, hw: &HardwareInfo) -> Result<(), CoreError>;
    fn dma_submit(&mut self, hw: &HardwareInfo, bytes: u64) -> Result<(), CoreError>;
}

/// Stable lifecycle contract implemented by every AWE-native or translated
/// driver adapter. This gives all source operating systems the same lifecycle.
pub trait DriverAdapter {
    fn identity(&self) -> DriverIdentity;
    fn probe(&mut self, hw: &HardwareInfo) -> Result<(), CoreError>;
    fn start(&mut self, hw: &HardwareInfo) -> Result<(), CoreError>;
    fn stop(&mut self, hw: &HardwareInfo) -> Result<(), CoreError>;
    fn remove(&mut self, hw: &HardwareInfo) -> Result<(), CoreError>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdapterState { New, Probed, Running, Stopped, Removed }

pub struct DriverSlot<A> {
    pub adapter: A,
    pub state: AdapterState,
}

impl<A: DriverAdapter> DriverSlot<A> {
    pub const fn new(adapter: A) -> Self { Self { adapter, state: AdapterState::New } }

    pub fn dispatch<H: HardwareAbstraction>(
        &mut self,
        action: DriverAction,
        hw: &HardwareInfo,
        _hal: &mut H,
    ) -> Result<(), CoreError> {
        if !hw.valid() || !self.adapter.identity().matches(hw) {
            return Err(CoreError::InvalidDevice);
        }
        match action {
            DriverAction::Probe => {
                self.adapter.probe(hw)?;
                self.state = AdapterState::Probed;
                Ok(())
            }
            DriverAction::Start => {
                if self.state != AdapterState::Probed && self.state != AdapterState::Stopped {
                    return Err(CoreError::NotBound);
                }
                self.adapter.start(hw)?;
                self.state = AdapterState::Running;
                Ok(())
            }
            DriverAction::Stop => {
                if self.state != AdapterState::Running { return Err(CoreError::NotBound); }
                self.adapter.stop(hw)?;
                self.state = AdapterState::Stopped;
                Ok(())
            }
            DriverAction::Remove => {
                if self.state == AdapterState::Running { return Err(CoreError::PolicyDenied); }
                self.adapter.remove(hw)?;
                self.state = AdapterState::Removed;
                Ok(())
            }
            DriverAction::Bind => Err(CoreError::InvalidRequest),
        }
    }
}

/// Linux adapter boundary. Native Linux code is not called here directly;
/// the implementation supplies a translation into `DriverAdapter`.
pub trait LinuxDriverAdapter: DriverAdapter {
    fn linux_api_version(&self) -> u32;
    fn linux_module_name(&self) -> &'static str;
}

/// Windows WDM/KMDF compatibility boundary.
pub trait WindowsDriverAdapter: DriverAdapter {
    fn windows_api_version(&self) -> u32;
    fn windows_driver_name(&self) -> &'static str;
}

/// Android HAL/vendor-driver compatibility boundary.
pub trait AndroidDriverAdapter: DriverAdapter {
    fn android_hal_version(&self) -> u32;
    fn android_interface_name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::{DeviceId, DriverAction};

    struct MockHal;
    impl HardwareAbstraction for MockHal {
        fn mmio_read32(&self, _hw: &HardwareInfo, _offset: u64) -> Result<u32, CoreError> { Ok(0) }
        fn mmio_write32(&mut self, _hw: &HardwareInfo, _offset: u64, _value: u32) -> Result<(), CoreError> { Ok(()) }
        fn irq_ack(&mut self, _hw: &HardwareInfo) -> Result<(), CoreError> { Ok(()) }
        fn dma_submit(&mut self, _hw: &HardwareInfo, _bytes: u64) -> Result<(), CoreError> { Ok(()) }
    }

    struct MockDriver;
    impl DriverAdapter for MockDriver {
        fn identity(&self) -> DriverIdentity { DriverIdentity { os: DriverOs::Linux, abi: DriverAbi::LinuxKmod, api_version: 1, vendor: 0x1234, device: 0x5678, signed: true } }
        fn probe(&mut self, _hw: &HardwareInfo) -> Result<(), CoreError> { Ok(()) }
        fn start(&mut self, _hw: &HardwareInfo) -> Result<(), CoreError> { Ok(()) }
        fn stop(&mut self, _hw: &HardwareInfo) -> Result<(), CoreError> { Ok(()) }
        fn remove(&mut self, _hw: &HardwareInfo) -> Result<(), CoreError> { Ok(()) }
    }

    fn hw() -> HardwareInfo {
        HardwareInfo { id: DeviceId { vendor: 0x1234, device: 0x5678, class: 0x0200, revision: 1 }, mmio_base: 0x1000, mmio_length: 0x1000, irq: 5, dma_bits: 64 }
    }

    #[test]
    fn lifecycle_is_enforced() {
        let mut slot = DriverSlot::new(MockDriver);
        let mut hal = MockHal;
        assert_eq!(slot.dispatch(DriverAction::Start, &hw(), &mut hal), Err(CoreError::NotBound));
        slot.dispatch(DriverAction::Probe, &hw(), &mut hal).unwrap();
        slot.dispatch(DriverAction::Start, &hw(), &mut hal).unwrap();
        slot.dispatch(DriverAction::Stop, &hw(), &mut hal).unwrap();
        slot.dispatch(DriverAction::Remove, &hw(), &mut hal).unwrap();
        assert_eq!(slot.state, AdapterState::Removed);
    }

    #[test]
    fn wrong_device_is_rejected() {
        let mut slot = DriverSlot::new(MockDriver);
        let mut hal = MockHal;
        let mut bad = hw();
        bad.id.device = 0x9999;
        assert_eq!(slot.dispatch(DriverAction::Probe, &bad, &mut hal), Err(CoreError::InvalidDevice));
    }
}

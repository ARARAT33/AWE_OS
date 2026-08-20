#![no_std]
use super::contract::DeviceContract;
use super::core::{
    AdapterState, CoreError, DriverSlot, HardwareAbstraction, HardwareInfo,
    WindowsDriverAdapter,
};
use super::universal::{DriverAbi, DriverOs};
pub struct WindowsLayer<A> {
    pub slot: DriverSlot<A>,
}
impl<A: WindowsDriverAdapter> WindowsLayer<A> {
    pub const fn new(adapter: A) -> Self {
        Self {
            slot: DriverSlot::new(adapter),
        }
    }
    pub fn validate(&self, hw: &HardwareInfo) -> Result<(), CoreError> {
        let id = self.slot.adapter.identity();
        if id.os != DriverOs::Windows || id.abi != DriverAbi::WindowsCompat {
            return Err(CoreError::UnsupportedAbi);
        }
        if !id.signed {
            return Err(CoreError::PolicyDenied);
        }
        let api = self.slot.adapter.windows_api_version();
        if id.api_version == 0
            || api == 0
            || api > u32::from(u16::MAX)
            || id.api_version != api as u16
        {
            return Err(CoreError::InvalidRequest);
        }
        let name = self.slot.adapter.windows_driver_name();
        if name.len() < 3 || !name.as_bytes().contains(&b'-') {
            return Err(CoreError::InvalidRequest);
        }
        if !id.matches(hw) || !hw.valid() || hw.irq == u32::MAX {
            return Err(CoreError::InvalidDevice);
        }
        Ok(())
    }
    pub fn validate_contract<const M: usize>(
        &self,
        hw: &HardwareInfo,
        contract: &DeviceContract<M>,
    ) -> Result<(), CoreError> {
        self.validate(hw)?;
        if !contract.valid() {
            return Err(CoreError::InvalidRequest);
        }
        if contract.vendor != hw.id.vendor
            || contract.device != hw.id.device
            || contract.class_code != hw.id.class
        {
            return Err(CoreError::InvalidDevice);
        }
        let end = hw
            .mmio_base
            .checked_add(hw.mmio_length)
            .ok_or(CoreError::MmioDenied)?;
        if !contract.allows_mmio_range(hw.mmio_base, hw.mmio_length)
            || contract.mmio.iter().flatten().any(|r| {
                r.base.checked_add(r.length).is_none()
                    || r.base < hw.mmio_base
                    || r.base.checked_add(r.length).unwrap() > end
            })
        {
            return Err(CoreError::MmioDenied);
        }
        if hw.dma_bits < 32 || hw.dma_bits > contract.dma.address_bits {
            return Err(CoreError::DmaDenied);
        }
        Ok(())
    }
    pub fn can_probe(&self, hw: &HardwareInfo) -> bool {
        self.validate(hw).is_ok() && self.slot.state == AdapterState::New
    }
    pub fn probe(&mut self, hw: &HardwareInfo) -> Result<(), CoreError> {
        self.validate(hw)?;
        if self.slot.state != AdapterState::New {
            return Err(CoreError::NotBound);
        }
        self.slot
            .adapter
            .probe(hw)
            .map_err(|_| CoreError::ProbeFailed)?;
        self.slot.state = AdapterState::Probed;
        Ok(())
    }
    pub fn start(&mut self, hw: &HardwareInfo) -> Result<(), CoreError> {
        self.validate(hw)?;
        if self.slot.state != AdapterState::Probed && self.slot.state != AdapterState::Stopped {
            return Err(CoreError::NotBound);
        }
        self.slot
            .adapter
            .start(hw)
            .map_err(|_| CoreError::StartFailed)?;
        self.slot.state = AdapterState::Running;
        Ok(())
    }
    pub fn stop(&mut self, hw: &HardwareInfo) -> Result<(), CoreError> {
        if self.slot.state != AdapterState::Running {
            return Err(CoreError::NotBound);
        }
        self.slot
            .adapter
            .stop(hw)
            .map_err(|_| CoreError::PolicyDenied)?;
        self.slot.state = AdapterState::Stopped;
        Ok(())
    }
    pub fn remove(&mut self, hw: &HardwareInfo) -> Result<(), CoreError> {
        if self.slot.state == AdapterState::Running {
            return Err(CoreError::PolicyDenied);
        }
        if self.slot.state == AdapterState::Removed {
            return Err(CoreError::NotBound);
        }
        self.slot
            .adapter
            .remove(hw)
            .map_err(|_| CoreError::PolicyDenied)?;
        self.slot.state = AdapterState::Removed;
        Ok(())
    }
    pub fn irq_ack<H: HardwareAbstraction>(
        &mut self,
        hw: &HardwareInfo,
        hal: &mut H,
    ) -> Result<(), CoreError> {
        if self.slot.state != AdapterState::Running {
            return Err(CoreError::NotBound);
        }
        if hw.irq == u32::MAX {
            return Err(CoreError::IrqDenied);
        }
        hal.irq_ack(hw)
    }
    pub fn shutdown<H: HardwareAbstraction>(
        &mut self,
        hw: &HardwareInfo,
        hal: &mut H,
    ) -> Result<(), CoreError> {
        if self.slot.state == AdapterState::Running {
            self.irq_ack(hw, hal)?;
            self.stop(hw)?;
        }
        if self.slot.state == AdapterState::Stopped || self.slot.state == AdapterState::Probed {
            self.remove(hw)?;
            Ok(())
        } else {
            Err(CoreError::NotBound)
        }
    }
    pub fn recover<H: HardwareAbstraction>(
        &mut self,
        hw: &HardwareInfo,
        hal: &mut H,
    ) -> Result<(), CoreError> {
        self.validate(hw)?;
        match self.slot.state {
            AdapterState::New => {
                self.probe(hw)?;
                self.start(hw)?
            }
            AdapterState::Probed => self.start(hw)?,
            AdapterState::Running => {
                self.irq_ack(hw, hal)?;
                self.stop(hw)?;
                self.start(hw)?
            }
            AdapterState::Stopped => self.start(hw)?,
            AdapterState::Removed => return Err(CoreError::NotBound),
        }
        Ok(())
    }
    pub fn io_cycle<H: HardwareAbstraction>(
        &mut self,
        hw: &HardwareInfo,
        hal: &mut H,
        offset: u64,
        value: u32,
        dma_bytes: u64,
        address_bits: u8,
    ) -> Result<u32, CoreError> {
        if self.slot.state == AdapterState::New {
            self.probe(hw)?
        }
        if self.slot.state == AdapterState::Probed {
            self.start(hw)?
        }
        self.slot.mmio_write32(hw, hal, offset, value)?;
        let read = self.slot.mmio_read32(hw, hal, offset)?;
        self.slot.dma_submit(hw, hal, dma_bytes, address_bits)?;
        Ok(read)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn io_cycle_contract<H: HardwareAbstraction, const M: usize>(
        &mut self,
        hw: &HardwareInfo,
        contract: &DeviceContract<M>,
        hal: &mut H,
        offset: u64,
        value: u32,
        dma_bytes: u64,
        address_bits: u8,
    ) -> Result<u32, CoreError> {
        self.validate_contract(hw, contract)?;
        let absolute = hw
            .mmio_base
            .checked_add(offset)
            .ok_or(CoreError::MmioDenied)?;
        if !contract.allows_mmio_range(absolute, 4) {
            return Err(CoreError::MmioDenied);
        }
        if self.slot.state == AdapterState::New {
            self.probe(hw)?
        }
        if self.slot.state == AdapterState::Probed {
            self.start(hw)?
        }
        if !contract.allows_dma(dma_bytes, address_bits) {
            return Err(CoreError::DmaDenied);
        }
        self.slot
            .mmio_write32_contract_base(hw, contract, hal, offset, value)?;
        let read = self
            .slot
            .mmio_read32_contract_base(hw, contract, hal, offset)?;
        self.slot
            .dma_submit_contract(hw, contract, hal, dma_bytes, address_bits)?;
        Ok(read)
    }
}

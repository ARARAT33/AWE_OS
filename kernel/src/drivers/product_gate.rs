#![no_std]

use super::contract::DeviceContract;
use super::core::{DriverIdentity, HardwareInfo};
use super::universal::{validate_request, DriverRequest, DriverAction, DriverError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProductGateError {
    Request(DriverError),
    InvalidHardware,
    ContractMismatch,
    IdentityMismatch,
    UnsupportedAction,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProductGate {
    pub min_score: u16,
}

impl ProductGate {
    pub const fn strict() -> Self { Self { min_score: 90 } }

    pub fn validate<const M: usize>(
        &self,
        request: &DriverRequest,
        identity: &DriverIdentity,
        hardware: &HardwareInfo,
        contract: &DeviceContract<M>,
    ) -> Result<u16, ProductGateError> {
        let result = validate_request(request);
        if !result.accepted || result.score < self.min_score {
            return Err(ProductGateError::Request(result.error.unwrap_or(DriverError::PolicyDenied)));
        }
        if matches!(request.action, DriverAction::Bind | DriverAction::Probe | DriverAction::Start)
            && !hardware.valid() {
            return Err(ProductGateError::InvalidHardware);
        }
        if identity.os != request.os || identity.abi != request.abi || !identity.matches(hardware) {
            return Err(ProductGateError::IdentityMismatch);
        }
        if !contract.valid() || contract.vendor != hardware.id.vendor || contract.device != hardware.id.device {
            return Err(ProductGateError::ContractMismatch);
        }
        if matches!(request.action, DriverAction::Start | DriverAction::Bind) && !identity.signed {
            return Err(ProductGateError::IdentityMismatch);
        }
        if request.action == DriverAction::Remove && !identity.signed {
            return Err(ProductGateError::UnsupportedAction);
        }
        Ok(result.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::bus::DeviceId;
    use super::super::contract::{DmaPolicy, InterruptMode, MmioRegion};
    use super::super::universal::{DriverAbi, DriverOs};

    fn hardware() -> HardwareInfo {
        HardwareInfo { id: DeviceId { vendor: 1, device: 2, class: 0, revision: 1 }, mmio_base: 0x1000, mmio_length: 0x100, irq: 5, dma_bits: 64 }
    }

    fn identity() -> DriverIdentity {
        DriverIdentity { os: DriverOs::Windows, abi: DriverAbi::WindowsCompat, api_version: 1, vendor: 1, device: 2, signed: true }
    }

    fn contract() -> DeviceContract<1> {
        DeviceContract { vendor: 1, device: 2, class_code: 0, mmio: [Some(MmioRegion { base: 0x1000, length: 0x100 })], interrupt: InterruptMode::Msi, dma: DmaPolicy { max_bytes: 4096, address_bits: 64, coherent: true } }
    }

    fn request() -> DriverRequest {
        DriverRequest { device: DeviceId(0x0001_0002), os: DriverOs::Windows, abi: DriverAbi::WindowsCompat, action: DriverAction::Start, version: 1, signed: true }
    }

    #[test]
    fn strict_gate_accepts_consistent_driver() {
        assert!(ProductGate::strict().validate(&request(), &identity(), &hardware(), &contract()).is_ok());
    }

    #[test]
    fn gate_rejects_identity_mismatch() {
        let mut id = identity();
        id.device = 9;
        assert_eq!(ProductGate::strict().validate(&request(), &id, &hardware(), &contract()), Err(ProductGateError::IdentityMismatch));
    }

    #[test]
    fn gate_rejects_contract_mismatch() {
        let mut c = contract();
        c.vendor = 9;
        assert_eq!(ProductGate::strict().validate(&request(), &identity(), &hardware(), &c), Err(ProductGateError::ContractMismatch));
    }
}

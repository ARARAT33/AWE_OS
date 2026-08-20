#![no_std]
use super::contract::DeviceContract;
use super::core::{DriverIdentity, HardwareInfo};
use super::universal::{DriverAction, DriverError, DriverRequest, validate_request};
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
    pub const fn strict() -> Self {
        Self { min_score: 90 }
    }
    pub fn validate<const M: usize>(
        &self,
        request: &DriverRequest,
        identity: &DriverIdentity,
        hardware: &HardwareInfo,
        contract: &DeviceContract<M>,
    ) -> Result<u16, ProductGateError> {
        let result = validate_request(request);
        if !result.accepted || result.score < self.min_score {
            return Err(ProductGateError::Request(
                result.error.unwrap_or(DriverError::PolicyDenied),
            ));
        }
        if matches!(
            request.action,
            DriverAction::Bind | DriverAction::Probe | DriverAction::Start
        ) && !hardware.valid()
        {
            return Err(ProductGateError::InvalidHardware);
        }
        if identity.os != request.os || identity.abi != request.abi || !identity.matches(hardware) {
            return Err(ProductGateError::IdentityMismatch);
        }
        if !contract.valid()
            || contract.vendor != hardware.id.vendor
            || contract.device != hardware.id.device
        {
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

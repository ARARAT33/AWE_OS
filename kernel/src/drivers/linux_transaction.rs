#![no_std]
use super::bus::DeviceId;
use super::contract::HardwareResource;
use super::linux_install::{InstallError, InstallPlan, plan};
use super::linux_package::LinuxPackageHeader;
use super::linux_resolver::LinuxCandidate;
use super::linux_runtime::LinuxRuntime;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionState {
    Planned,
    Prepared,
    Activating,
    Activated,
    RollbackRequired,
    RolledBack,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionError {
    Install(InstallError),
    InvalidState,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverTransaction {
    pub state: TransactionState,
    pub plan: InstallPlan,
    pub activated_count: usize,
}
impl DriverTransaction {
    pub fn prepare(
        device: DeviceId,
        candidates: &[LinuxCandidate],
        runtime: &LinuxRuntime,
        header: LinuxPackageHeader,
        resources: HardwareResource,
    ) -> Result<Self, TransactionError> {
        let plan = plan(device, candidates, runtime, header, resources)
            .map_err(TransactionError::Install)?;
        Ok(Self {
            state: TransactionState::Prepared,
            plan,
            activated_count: 0,
        })
    }
    pub const fn begin_activation(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Prepared) {
            return Err(TransactionError::InvalidState);
        }
        Ok(Self {
            state: TransactionState::Activating,
            ..self
        })
    }
    pub const fn mark_activated(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Activating) {
            return Err(TransactionError::InvalidState);
        }
        Ok(Self {
            state: TransactionState::Activated,
            activated_count: self.activated_count + 1,
            ..self
        })
    }
    pub const fn require_rollback(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Activating) {
            return Err(TransactionError::InvalidState);
        }
        Ok(Self {
            state: TransactionState::RollbackRequired,
            ..self
        })
    }
    pub const fn rollback(self) -> Result<Self, TransactionError> {
        if !matches!(
            self.state,
            TransactionState::RollbackRequired
                | TransactionState::Prepared
                | TransactionState::Activated
        ) {
            return Err(TransactionError::InvalidState);
        }
        Ok(Self {
            state: TransactionState::RolledBack,
            activated_count: 0,
            ..self
        })
    }
    pub fn activate(self) -> Result<Self, TransactionError> {
        self.begin_activation()?.mark_activated()
    }
}

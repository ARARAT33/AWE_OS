#![no_std]

use super::bus::DeviceId;
use super::contract::HardwareResource;
use super::linux_install::{plan, InstallError, InstallPlan};
use super::linux_package::LinuxPackageHeader;
use super::linux_resolver::LinuxCandidate;
use super::linux_runtime::LinuxRuntime;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionState { Planned, Prepared, Activating, Activated, RollbackRequired, RolledBack }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionError { Install(InstallError), InvalidState }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverTransaction {
    pub state: TransactionState,
    pub plan: InstallPlan,
    pub activated_count: usize,
}

impl DriverTransaction {
    pub fn prepare(device: DeviceId, candidates: &[LinuxCandidate], runtime: &LinuxRuntime, header: LinuxPackageHeader, resources: HardwareResource) -> Result<Self, TransactionError> {
        let plan = plan(device, candidates, runtime, header, resources).map_err(TransactionError::Install)?;
        Ok(Self { state: TransactionState::Prepared, plan, activated_count: 0 })
    }

    pub const fn begin_activation(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Prepared) { return Err(TransactionError::InvalidState); }
        Ok(Self { state: TransactionState::Activating, ..self })
    }

    pub const fn mark_activated(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Activating) { return Err(TransactionError::InvalidState); }
        Ok(Self { state: TransactionState::Activated, activated_count: self.activated_count + 1, ..self })
    }

    pub const fn require_rollback(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Activating) { return Err(TransactionError::InvalidState); }
        Ok(Self { state: TransactionState::RollbackRequired, ..self })
    }

    pub const fn rollback(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::RollbackRequired | TransactionState::Prepared | TransactionState::Activated) {
            return Err(TransactionError::InvalidState);
        }
        Ok(Self { state: TransactionState::RolledBack, activated_count: 0, ..self })
    }

    pub const fn activate(self) -> Result<Self, TransactionError> { self.begin_activation()?.mark_activated() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::linux_package::{LinuxDriverDescriptor, LDRIVER_MAGIC};
    const DEV: DeviceId = DeviceId { vendor: 0x8086, device: 0x100e, class: 0x0200, revision: 1 };
    const RES: HardwareResource = HardwareResource { mmio_base: 0x1000, mmio_length: 0x1000, dma_mask: u64::MAX, irq: 11 };
    const DESC: LinuxDriverDescriptor = LinuxDriverDescriptor { vendor: 0x8086, device: 0x100e, class: 0x0200, api_version: 6, module_hash: 1, signed: true };
    const HDR: LinuxPackageHeader = LinuxPackageHeader { magic: LDRIVER_MAGIC, format_version: 1, descriptor_size: core::mem::size_of::<LinuxDriverDescriptor>() as u16, payload_size: 4096, checksum: 1 };

    fn prepared() -> DriverTransaction {
        let candidate = LinuxCandidate { descriptor: DESC, priority: 10 };
        DriverTransaction::prepare(DEV, &[candidate], &LinuxRuntime::new(6), HDR, RES).unwrap()
    }

    #[test]
    fn transaction_can_prepare_activate_and_rollback() {
        let tx = prepared();
        assert_eq!(tx.state, TransactionState::Prepared);
        let tx = tx.activate().unwrap();
        assert_eq!(tx.state, TransactionState::Activated);
        assert_eq!(tx.rollback().unwrap().state, TransactionState::RolledBack);
    }

    #[test]
    fn activation_failure_enters_rollback_required() {
        let tx = prepared().begin_activation().unwrap();
        let tx = tx.require_rollback().unwrap();
        assert_eq!(tx.state, TransactionState::RollbackRequired);
        assert_eq!(tx.rollback().unwrap().state, TransactionState::RolledBack);
    }
}

#![no_std]

use super::bus::DeviceId;
use super::contract::HardwareResource;
use super::linux_install::{plan, InstallError, InstallPlan};
use super::linux_package::LinuxPackageHeader;
use super::linux_resolver::LinuxCandidate;
use super::linux_runtime::LinuxRuntime;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionState { Planned, Prepared, Activated, RolledBack }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TransactionError { Install(InstallError), InvalidState }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriverTransaction {
    pub state: TransactionState,
    pub plan: InstallPlan,
}

impl DriverTransaction {
    pub fn prepare(device: DeviceId, candidates: &[LinuxCandidate], runtime: &LinuxRuntime, header: LinuxPackageHeader, resources: HardwareResource) -> Result<Self, TransactionError> {
        let plan = plan(device, candidates, runtime, header, resources).map_err(TransactionError::Install)?;
        Ok(Self { state: TransactionState::Prepared, plan })
    }

    pub const fn activate(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Prepared) { return Err(TransactionError::InvalidState); }
        Ok(Self { state: TransactionState::Activated, ..self })
    }

    pub const fn rollback(self) -> Result<Self, TransactionError> {
        if !matches!(self.state, TransactionState::Activated | TransactionState::Prepared) { return Err(TransactionError::InvalidState); }
        Ok(Self { state: TransactionState::RolledBack, ..self })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::linux_package::{LinuxDriverDescriptor, LDRIVER_MAGIC};
    const DEV: DeviceId = DeviceId { vendor: 0x8086, device: 0x100e, class: 0x0200, revision: 1 };
    const RES: HardwareResource = HardwareResource { mmio_base: 0x1000, mmio_length: 0x1000, dma_mask: u64::MAX, irq: 11 };
    const DESC: LinuxDriverDescriptor = LinuxDriverDescriptor { vendor: 0x8086, device: 0x100e, class: 0x0200, api_version: 6, module_hash: 1, signed: true };
    const HDR: LinuxPackageHeader = LinuxPackageHeader { magic: LDRIVER_MAGIC, format_version: 1, descriptor_size: core::mem::size_of::<LinuxDriverDescriptor>() as u16, payload_size: 4096, checksum: 1 };
    #[test]
    fn transaction_can_prepare_activate_and_rollback() {
        let candidate = LinuxCandidate { descriptor: DESC, priority: 10 };
        let tx = DriverTransaction::prepare(DEV, &[candidate], &LinuxRuntime::new(6), HDR, RES).unwrap();
        assert_eq!(tx.state, TransactionState::Prepared);
        let tx = tx.activate().unwrap();
        assert_eq!(tx.state, TransactionState::Activated);
        assert_eq!(tx.rollback().unwrap().state, TransactionState::RolledBack);
    }
}

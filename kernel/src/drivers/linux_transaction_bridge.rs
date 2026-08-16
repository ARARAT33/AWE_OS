#![no_std]

use super::linux_transaction::{DriverTransaction, TransactionError, TransactionState};
use super::linux_transaction_orchestrator::{ActivationOrchestrator, OrchestratorError, OrchestratorState};
use super::linux_driver_ops::{DriverLifecycle, DriverOp, DriverOpError, DriverState};
use super::linux_dependency::Dependency;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BridgeError { Transaction(TransactionError), Orchestrator(OrchestratorError), Lifecycle(DriverOpError), StateMismatch }

pub struct TransactionBridge<const N: usize> {
    pub transaction: DriverTransaction,
    pub orchestrator: ActivationOrchestrator<N>,
    pub lifecycle: DriverLifecycle,
}

impl<const N: usize> TransactionBridge<N> {
    pub const fn new() -> Self { Self { transaction: DriverTransaction::new(), orchestrator: ActivationOrchestrator::new(), lifecycle: DriverLifecycle::new() } }

    pub fn prepare(&mut self, nodes: &[u64], deps: &[Dependency]) -> Result<(), BridgeError> {
        self.transaction.prepare().map_err(BridgeError::Transaction)?;
        self.orchestrator.prepare(nodes, deps).map_err(BridgeError::Orchestrator)?;
        self.lifecycle.apply(DriverOp::Probe, true).map_err(BridgeError::Lifecycle)?;
        if self.transaction.state != TransactionState::Prepared || self.orchestrator.state != OrchestratorState::Ready || self.lifecycle.state != DriverState::Probed { return Err(BridgeError::StateMismatch); }
        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), BridgeError> {
        self.transaction.begin_activation().map_err(BridgeError::Transaction)?;
        self.orchestrator.begin().map_err(BridgeError::Orchestrator)?;
        self.lifecycle.apply(DriverOp::Init, true).map_err(BridgeError::Lifecycle)?;
        if self.transaction.state != TransactionState::Activating || self.orchestrator.state != OrchestratorState::Activating || self.lifecycle.state != DriverState::Initialized { return Err(BridgeError::StateMismatch); }
        Ok(())
    }

    pub fn mark_activated(&mut self) -> Result<u64, BridgeError> {
        let node = self.orchestrator.mark_activated().map_err(BridgeError::Orchestrator)?;
        self.transaction.mark_activated().map_err(BridgeError::Transaction)?;
        if self.orchestrator.state == OrchestratorState::Activating && self.lifecycle.state == DriverState::Initialized { self.lifecycle.apply(DriverOp::Start, true).map_err(BridgeError::Lifecycle)?; }
        if self.orchestrator.state == OrchestratorState::Activated && (self.transaction.state != TransactionState::Activated || self.lifecycle.state != DriverState::Running) { return Err(BridgeError::StateMismatch); }
        Ok(node)
    }

    pub fn fail(&mut self, failed: DriverOp, rollback: &mut [u64]) -> Result<usize, BridgeError> {
        let n = self.orchestrator.fail(rollback).map_err(BridgeError::Orchestrator)?;
        self.transaction.require_rollback().map_err(BridgeError::Transaction)?;
        self.lifecycle.rollback_after_failure(failed).map_err(BridgeError::Lifecycle)?;
        if self.transaction.state != TransactionState::RollbackRequired || self.orchestrator.state != OrchestratorState::RollbackRequired { return Err(BridgeError::StateMismatch); }
        Ok(n)
    }

    pub fn complete_rollback(&mut self) -> Result<(), BridgeError> {
        self.transaction.rollback().map_err(BridgeError::Transaction)?;
        self.orchestrator.complete_rollback().map_err(BridgeError::Orchestrator)?;
        if self.lifecycle.state == DriverState::Running { self.lifecycle.apply(DriverOp::Stop, true).map_err(BridgeError::Lifecycle)?; }
        if self.lifecycle.state == DriverState::Stopped { self.lifecycle.apply(DriverOp::Remove, true).map_err(BridgeError::Lifecycle)?; }
        if self.transaction.state != TransactionState::RolledBack || self.orchestrator.state != OrchestratorState::RolledBack || self.lifecycle.state != DriverState::Removed { return Err(BridgeError::StateMismatch); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lifecycle_and_transaction_roll_back_together() {
        let mut b = TransactionBridge::<3>::new();
        let nodes = [1,2,3]; let deps = [Dependency { driver_hash:1, required_hash:2 }, Dependency { driver_hash:2, required_hash:3 }];
        b.prepare(&nodes,&deps).unwrap(); b.begin().unwrap(); b.mark_activated().unwrap();
        let mut rb=[0;1]; b.fail(DriverOp::Start,&mut rb).unwrap(); b.complete_rollback().unwrap();
        assert_eq!(b.transaction.state,TransactionState::RolledBack); assert_eq!(b.orchestrator.state,OrchestratorState::RolledBack); assert_eq!(b.lifecycle.state,DriverState::Removed);
    }
}

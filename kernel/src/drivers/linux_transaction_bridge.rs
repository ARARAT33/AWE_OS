#![no_std]

use super::linux_transaction::{DriverTransaction, TransactionError, TransactionState};
use super::linux_transaction_orchestrator::{ActivationOrchestrator, OrchestratorError, OrchestratorState};
use super::linux_dependency::Dependency;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BridgeError { Transaction(TransactionError), Orchestrator(OrchestratorError), StateMismatch }

/// Keeps the driver transaction and dependency orchestrator synchronized.
pub struct TransactionBridge<const N: usize> {
    pub transaction: DriverTransaction,
    pub orchestrator: ActivationOrchestrator<N>,
}

impl<const N: usize> TransactionBridge<N> {
    pub const fn new() -> Self {
        Self { transaction: DriverTransaction::new(), orchestrator: ActivationOrchestrator::new() }
    }

    pub fn prepare(&mut self, nodes: &[u64], deps: &[Dependency]) -> Result<(), BridgeError> {
        self.transaction.prepare().map_err(BridgeError::Transaction)?;
        self.orchestrator.prepare(nodes, deps).map_err(BridgeError::Orchestrator)?;
        if self.transaction.state != TransactionState::Prepared || self.orchestrator.state != OrchestratorState::Ready {
            return Err(BridgeError::StateMismatch);
        }
        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), BridgeError> {
        self.transaction.begin_activation().map_err(BridgeError::Transaction)?;
        self.orchestrator.begin().map_err(BridgeError::Orchestrator)?;
        if self.transaction.state != TransactionState::Activating || self.orchestrator.state != OrchestratorState::Activating {
            return Err(BridgeError::StateMismatch);
        }
        Ok(())
    }

    pub fn mark_activated(&mut self) -> Result<u64, BridgeError> {
        let node = self.orchestrator.mark_activated().map_err(BridgeError::Orchestrator)?;
        self.transaction.mark_activated().map_err(BridgeError::Transaction)?;
        if self.orchestrator.state == OrchestratorState::Activated {
            if self.transaction.state != TransactionState::Activated { return Err(BridgeError::StateMismatch); }
        }
        Ok(node)
    }

    pub fn fail(&mut self, rollback: &mut [u64]) -> Result<usize, BridgeError> {
        let n = self.orchestrator.fail(rollback).map_err(BridgeError::Orchestrator)?;
        self.transaction.require_rollback().map_err(BridgeError::Transaction)?;
        if self.transaction.state != TransactionState::RollbackRequired || self.orchestrator.state != OrchestratorState::RollbackRequired {
            return Err(BridgeError::StateMismatch);
        }
        Ok(n)
    }

    pub fn complete_rollback(&mut self) -> Result<(), BridgeError> {
        self.transaction.rollback().map_err(BridgeError::Transaction)?;
        self.orchestrator.complete_rollback().map_err(BridgeError::Orchestrator)?;
        if self.transaction.state != TransactionState::RolledBack || self.orchestrator.state != OrchestratorState::RolledBack {
            return Err(BridgeError::StateMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_keeps_both_state_machines_in_sync() {
        let mut bridge = TransactionBridge::<3>::new();
        let nodes = [1, 2, 3];
        let deps = [Dependency { driver_hash: 1, required_hash: 2 }, Dependency { driver_hash: 2, required_hash: 3 }];
        bridge.prepare(&nodes, &deps).unwrap();
        bridge.begin().unwrap();
        bridge.mark_activated().unwrap();
        bridge.mark_activated().unwrap();
        let mut rollback = [0; 2];
        bridge.fail(&mut rollback).unwrap();
        assert_eq!(rollback, [2, 3]);
        bridge.complete_rollback().unwrap();
        assert_eq!(bridge.transaction.state, TransactionState::RolledBack);
        assert_eq!(bridge.orchestrator.state, OrchestratorState::RolledBack);
    }

    #[test]
    fn successful_activation_finishes_both() {
        let mut bridge = TransactionBridge::<2>::new();
        let nodes = [1, 2];
        let deps = [Dependency { driver_hash: 1, required_hash: 2 }];
        bridge.prepare(&nodes, &deps).unwrap();
        bridge.begin().unwrap();
        bridge.mark_activated().unwrap();
        bridge.mark_activated().unwrap();
        assert_eq!(bridge.transaction.state, TransactionState::Activated);
        assert_eq!(bridge.orchestrator.state, OrchestratorState::Activated);
    }
}

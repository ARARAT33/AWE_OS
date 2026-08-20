#![no_std]

use super::linux_activation::{build_activation_order, validate_activation_order};
use super::linux_activation_rollback::activation_failed;
use super::linux_dependency::Dependency;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrchestratorError {
    Order,
    InvalidSequence,
    RollbackBufferFull,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OrchestratorState {
    Ready,
    Activating,
    Activated,
    RollbackRequired,
    RolledBack,
}

/// Allocation-free orchestration metadata for a multi-driver transaction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ActivationOrchestrator<const N: usize> {
    pub state: OrchestratorState,
    pub order: [u64; N],
    pub count: usize,
    pub activated: usize,
}

impl<const N: usize> ActivationOrchestrator<N> {
    pub const fn new() -> Self {
        Self {
            state: OrchestratorState::Ready,
            order: [0; N],
            count: 0,
            activated: 0,
        }
    }

    pub fn prepare(&mut self, nodes: &[u64], deps: &[Dependency]) -> Result<(), OrchestratorError> {
        if nodes.len() > N {
            return Err(OrchestratorError::Order);
        }
        self.count = build_activation_order(nodes, deps, &mut self.order)
            .map_err(|_| OrchestratorError::Order)?;
        validate_activation_order(&self.order[..self.count], deps)
            .map_err(|_| OrchestratorError::InvalidSequence)?;
        self.activated = 0;
        self.state = OrchestratorState::Ready;
        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), OrchestratorError> {
        if self.state != OrchestratorState::Ready {
            return Err(OrchestratorError::InvalidSequence);
        }
        self.state = OrchestratorState::Activating;
        Ok(())
    }

    pub fn mark_activated(&mut self) -> Result<u64, OrchestratorError> {
        if self.state != OrchestratorState::Activating || self.activated >= self.count {
            return Err(OrchestratorError::InvalidSequence);
        }
        let node = self.order[self.activated];
        self.activated += 1;
        if self.activated == self.count {
            self.state = OrchestratorState::Activated;
        }
        Ok(node)
    }

    pub fn fail(&mut self, rollback: &mut [u64]) -> Result<usize, OrchestratorError> {
        if self.state != OrchestratorState::Activating {
            return Err(OrchestratorError::InvalidSequence);
        }
        if rollback.len() < self.activated {
            return Err(OrchestratorError::RollbackBufferFull);
        }
        let n = activation_failed(&self.order[..self.count], self.activated, rollback)
            .map_err(|_| OrchestratorError::InvalidSequence)?;
        self.state = OrchestratorState::RollbackRequired;
        Ok(n)
    }

    pub fn complete_rollback(&mut self) -> Result<(), OrchestratorError> {
        if self.state != OrchestratorState::RollbackRequired {
            return Err(OrchestratorError::InvalidSequence);
        }
        self.activated = 0;
        self.state = OrchestratorState::RolledBack;
        Ok(())
    }
}

impl<const N: usize> Default for ActivationOrchestrator<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_rolls_back_only_successfully_activated_nodes() {
        let nodes = [10, 20, 30, 40];
        let deps = [
            Dependency {
                driver_hash: 10,
                required_hash: 20,
            },
            Dependency {
                driver_hash: 20,
                required_hash: 30,
            },
        ];
        let mut o = ActivationOrchestrator::<4>::new();
        o.prepare(&nodes, &deps).unwrap();
        o.begin().unwrap();
        assert!(o.mark_activated().is_ok());
        assert!(o.mark_activated().is_ok());
        let mut rollback = [0; 2];
        assert_eq!(o.fail(&mut rollback).unwrap(), 2);
        assert_eq!(rollback, [20, 30]);
        o.complete_rollback().unwrap();
        assert_eq!(o.state, OrchestratorState::RolledBack);
    }

    #[test]
    fn successful_sequence_reaches_activated() {
        let nodes = [1, 2];
        let deps = [Dependency {
            driver_hash: 1,
            required_hash: 2,
        }];
        let mut o = ActivationOrchestrator::<2>::new();
        o.prepare(&nodes, &deps).unwrap();
        o.begin().unwrap();
        assert!(o.mark_activated().is_ok());
        assert!(o.mark_activated().is_ok());
        assert_eq!(o.state, OrchestratorState::Activated);
    }
}

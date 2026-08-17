//! AWE 63.0 driver dependency, ownership and health contract.
//! Concrete hardware discovery/execution remains outside this contract.

use crate::{DriverId, DriverState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverDependency {
    pub driver: DriverId,
    pub requires: DriverId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceOwnership {
    pub driver: DriverId,
    pub mmio_bytes: u64,
    pub io_bytes: u64,
    pub dma_bytes: u64,
    pub interrupt_count: u16,
}

impl ResourceOwnership {
    pub fn within(self, budget: ResourceOwnership) -> bool {
        self.driver == budget.driver
            && self.mmio_bytes <= budget.mmio_bytes
            && self.io_bytes <= budget.io_bytes
            && self.dma_bytes <= budget.dma_bytes
            && self.interrupt_count <= budget.interrupt_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyError {
    SelfDependency,
    Cycle,
    MissingDependency,
    ResourceExceeded,
}

/// Bounded dependency table. It intentionally rejects self-dependencies and
/// simple transitive cycles before a driver is admitted to the service plane.
pub struct DependencyGraph<const N: usize> {
    edges: [Option<DriverDependency>; N],
    len: usize,
}

impl<const N: usize> DependencyGraph<N> {
    pub const fn new() -> Self {
        Self {
            edges: [None; N],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    fn requires(&self, driver: DriverId) -> Option<DriverId> {
        let mut i = 0;
        while i < self.len {
            if let Some(edge) = self.edges[i] {
                if edge.driver == driver {
                    return Some(edge.requires);
                }
            }
            i += 1;
        }
        None
    }

    pub fn add(&mut self, dependency: DriverDependency) -> Result<(), DependencyError> {
        if dependency.driver == dependency.requires {
            return Err(DependencyError::SelfDependency);
        }

        let mut cursor = dependency.requires;
        let mut steps = 0;
        while let Some(next) = self.requires(cursor) {
            if next == dependency.driver {
                return Err(DependencyError::Cycle);
            }
            cursor = next;
            steps += 1;
            if steps >= N {
                return Err(DependencyError::Cycle);
            }
        }

        if self.len == N {
            return Err(DependencyError::MissingDependency);
        }
        self.edges[self.len] = Some(dependency);
        self.len += 1;
        Ok(())
    }

    pub fn dependency_of(&self, driver: DriverId) -> Option<DriverId> {
        self.requires(driver)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverHealth {
    pub driver: DriverId,
    pub state: DriverState,
    pub consecutive_failures: u16,
    pub restart_count: u16,
    pub heartbeat_budget: u32,
}

impl DriverHealth {
    pub const fn new(driver: DriverId) -> Self {
        Self {
            driver,
            state: DriverState::Discovered,
            consecutive_failures: 0,
            restart_count: 0,
            heartbeat_budget: 0,
        }
    }

    pub const fn record_success(self) -> Self {
        Self {
            state: DriverState::Running,
            consecutive_failures: 0,
            heartbeat_budget: self.heartbeat_budget,
            ..self
        }
    }

    pub const fn record_failure(self) -> Self {
        Self {
            state: DriverState::Failed,
            consecutive_failures: self.consecutive_failures.saturating_add(1),
            heartbeat_budget: self.heartbeat_budget,
            ..self
        }
    }

    pub const fn can_restart(self, max_restarts: u16) -> bool {
        self.restart_count < max_restarts
    }

    pub const fn restarted(self) -> Self {
        Self {
            state: DriverState::Starting,
            restart_count: self.restart_count.saturating_add(1),
            consecutive_failures: 0,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_graph_rejects_self_and_cycles() {
        let mut graph: DependencyGraph<4> = DependencyGraph::new();
        assert_eq!(
            graph.add(DriverDependency {
                driver: DriverId(1),
                requires: DriverId(1),
            }),
            Err(DependencyError::SelfDependency)
        );
        assert_eq!(
            graph.add(DriverDependency {
                driver: DriverId(1),
                requires: DriverId(2),
            }),
            Ok(())
        );
        assert_eq!(
            graph.add(DriverDependency {
                driver: DriverId(2),
                requires: DriverId(3),
            }),
            Ok(())
        );
        assert_eq!(
            graph.add(DriverDependency {
                driver: DriverId(3),
                requires: DriverId(1),
            }),
            Err(DependencyError::Cycle)
        );
    }

    #[test]
    fn ownership_is_bounded_per_driver() {
        let budget = ResourceOwnership {
            driver: DriverId(5),
            mmio_bytes: 4096,
            io_bytes: 128,
            dma_bytes: 8192,
            interrupt_count: 4,
        };
        let granted = ResourceOwnership {
            driver: DriverId(5),
            mmio_bytes: 1024,
            io_bytes: 64,
            dma_bytes: 4096,
            interrupt_count: 2,
        };
        assert!(granted.within(budget));
        assert!(!ResourceOwnership {
            driver: DriverId(6),
            ..granted
        }
        .within(budget));
    }

    #[test]
    fn health_model_is_restartable_and_bounded() {
        let health = DriverHealth::new(DriverId(9)).record_failure().record_failure();
        assert_eq!(health.state, DriverState::Failed);
        assert_eq!(health.consecutive_failures, 2);
        assert!(health.can_restart(3));
        let restarted = health.restarted();
        assert_eq!(restarted.state, DriverState::Starting);
        assert_eq!(restarted.restart_count, 1);
        assert_eq!(restarted.consecutive_failures, 0);
    }
}

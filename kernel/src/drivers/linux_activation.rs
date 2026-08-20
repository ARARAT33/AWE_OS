#![no_std]

use super::linux_dependency::Dependency;
use super::linux_dependency_order::{OrderError, topological_order};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActivationError {
    Order(OrderError),
    OutputFull,
    InvalidSequence,
}

/// Builds a deterministic activation sequence from dependency metadata.
/// The returned order always places dependencies before their dependants.
pub fn build_activation_order(
    nodes: &[u64],
    deps: &[Dependency],
    out: &mut [u64],
) -> Result<usize, ActivationError> {
    topological_order(nodes, deps, out).map_err(ActivationError::Order)
}

/// Validates an externally supplied activation sequence against the dependency graph.
pub fn validate_activation_order(
    order: &[u64],
    deps: &[Dependency],
) -> Result<(), ActivationError> {
    for dep in deps {
        let driver_pos = order
            .iter()
            .position(|n| *n == dep.driver_hash)
            .ok_or(ActivationError::InvalidSequence)?;
        let required_pos = order
            .iter()
            .position(|n| *n == dep.required_hash)
            .ok_or(ActivationError::InvalidSequence)?;
        if required_pos >= driver_pos {
            return Err(ActivationError::InvalidSequence);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_dependency_first_order() {
        let nodes = [1, 2, 3];
        let deps = [
            Dependency {
                driver_hash: 1,
                required_hash: 2,
            },
            Dependency {
                driver_hash: 2,
                required_hash: 3,
            },
        ];
        let mut order = [0; 3];
        let count = build_activation_order(&nodes, &deps, &mut order).unwrap();
        assert_eq!(count, 3);
        assert!(validate_activation_order(&order, &deps).is_ok());
    }

    #[test]
    fn rejects_wrong_order() {
        let deps = [Dependency {
            driver_hash: 1,
            required_hash: 2,
        }];
        assert_eq!(
            validate_activation_order(&[1, 2], &deps),
            Err(ActivationError::InvalidSequence)
        );
        assert!(validate_activation_order(&[2, 1], &deps).is_ok());
    }
}

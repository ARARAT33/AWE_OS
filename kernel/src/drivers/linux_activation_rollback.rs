#![no_std]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RollbackError {
    InvalidOrder,
    RollbackFailed,
}

/// Tracks successfully activated drivers and returns the exact reverse-order
/// sequence required for safe rollback. No heap allocation is used.
pub fn build_rollback_order(
    activation_order: &[u64],
    out: &mut [u64],
) -> Result<usize, RollbackError> {
    if out.len() < activation_order.len() {
        return Err(RollbackError::InvalidOrder);
    }
    for (i, node) in activation_order.iter().rev().enumerate() {
        out[i] = *node;
    }
    Ok(activation_order.len())
}

/// Records an activation result. A failure means callers must rollback all
/// previously activated drivers in reverse activation order.
pub fn activation_failed(
    activation_order: &[u64],
    activated_count: usize,
    rollback_out: &mut [u64],
) -> Result<usize, RollbackError> {
    if activated_count > activation_order.len() || rollback_out.len() < activated_count {
        return Err(RollbackError::InvalidOrder);
    }
    for i in 0..activated_count {
        rollback_out[i] = activation_order[activated_count - 1 - i];
    }
    Ok(activated_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_is_reverse_order() {
        let order = [10, 20, 30, 40];
        let mut rollback = [0; 4];
        assert_eq!(build_rollback_order(&order, &mut rollback), Ok(4));
        assert_eq!(rollback, [40, 30, 20, 10]);
    }

    #[test]
    fn failed_fourth_driver_rolls_back_first_three() {
        let order = [10, 20, 30, 40, 50];
        let mut rollback = [0; 3];
        assert_eq!(activation_failed(&order, 3, &mut rollback), Ok(3));
        assert_eq!(rollback, [30, 20, 10]);
    }

    #[test]
    fn rejects_invalid_count() {
        let order = [1, 2];
        let mut rollback = [0; 2];
        assert_eq!(
            activation_failed(&order, 3, &mut rollback),
            Err(RollbackError::InvalidOrder)
        );
    }
}

#![no_std]

/// Monotonic image-version policy. Persistent storage integration belongs to
/// the platform adapter; this layer only defines the safe comparison rule.
pub fn accept(candidate: u64, minimum_allowed: u64) -> bool {
    candidate >= minimum_allowed
}

pub fn newer(candidate: u64, current: u64) -> bool {
    candidate > current
}

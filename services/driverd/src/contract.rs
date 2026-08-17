#![no_std]

//! AWE 62.5 driver-service contract.
//!
//! This freezes the driver lifecycle and manifest boundary without implementing
//! concrete PCI/ACPI/VirtIO hardware execution, which remains reserved for the
//! 65% checkpoint.

use crate::{DriverClass, DriverId, DriverState};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverLifecycle {
    Discover = 0,
    Identify = 1,
    Probe = 2,
    Bind = 3,
    Initialize = 4,
    Run = 5,
    Suspend = 6,
    Resume = 7,
    Stop = 8,
    Remove = 9,
    Recover = 10,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverTrust {
    Unverified = 0,
    Verified = 1,
    Revoked = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverManifest {
    pub id: DriverId,
    pub class: DriverClass,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub architecture_mask: u64,
    pub capability_mask: u64,
    pub trust: DriverTrust,
}

impl DriverManifest {
    pub const fn new(id: DriverId, class: DriverClass, abi_major: u16, abi_minor: u16, architecture_mask: u64, capability_mask: u64, trust: DriverTrust) -> Self {
        Self { id, class, abi_major, abi_minor, architecture_mask, capability_mask, trust }
    }
    pub const fn targets(self, architecture_bit: u64) -> bool { self.architecture_mask & architecture_bit != 0 }
    pub const fn declares(self, capability_bit: u64) -> bool { self.capability_mask & capability_bit != 0 }
    pub const fn is_trusted_for_execution(self) -> bool { matches!(self.trust, DriverTrust::Verified) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleError { InvalidTransition, Untrusted, AbiMismatch }

pub const fn transition(current: DriverLifecycle, next: DriverLifecycle, trusted: bool) -> Result<(), LifecycleError> {
    if !trusted { return Err(LifecycleError::Untrusted); }
    let valid = matches!((current, next),
        (DriverLifecycle::Discover, DriverLifecycle::Identify)
        | (DriverLifecycle::Identify, DriverLifecycle::Probe)
        | (DriverLifecycle::Probe, DriverLifecycle::Bind)
        | (DriverLifecycle::Bind, DriverLifecycle::Initialize)
        | (DriverLifecycle::Initialize, DriverLifecycle::Run)
        | (DriverLifecycle::Run, DriverLifecycle::Suspend)
        | (DriverLifecycle::Suspend, DriverLifecycle::Resume)
        | (DriverLifecycle::Resume, DriverLifecycle::Run)
        | (DriverLifecycle::Run, DriverLifecycle::Stop)
        | (DriverLifecycle::Stop, DriverLifecycle::Remove)
        | (DriverLifecycle::Remove, DriverLifecycle::Recover)
        | (DriverLifecycle::Recover, DriverLifecycle::Identify));
    if valid { Ok(()) } else { Err(LifecycleError::InvalidTransition) }
}

pub const fn lifecycle_maps_to_state(step: DriverLifecycle) -> DriverState {
    match step {
        DriverLifecycle::Discover | DriverLifecycle::Identify | DriverLifecycle::Probe | DriverLifecycle::Bind => DriverState::Discovered,
        DriverLifecycle::Initialize => DriverState::Starting,
        DriverLifecycle::Run | DriverLifecycle::Resume | DriverLifecycle::Recover => DriverState::Running,
        DriverLifecycle::Suspend | DriverLifecycle::Stop => DriverState::Stopping,
        DriverLifecycle::Remove => DriverState::Quarantined,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_accepts_only_declared_transitions() {
        assert_eq!(transition(DriverLifecycle::Discover, DriverLifecycle::Identify, true), Ok(()));
        assert_eq!(transition(DriverLifecycle::Discover, DriverLifecycle::Run, true), Err(LifecycleError::InvalidTransition));
    }

    #[test]
    fn untrusted_driver_cannot_enter_execution_path() {
        assert_eq!(transition(DriverLifecycle::Initialize, DriverLifecycle::Run, false), Err(LifecycleError::Untrusted));
    }

    #[test]
    fn manifest_exposes_architecture_and_capability_contract() {
        let manifest = DriverManifest::new(DriverId(5), DriverClass::Network, 1, 2, 0b1010, 0b0110, DriverTrust::Verified);
        assert!(manifest.targets(0b0010));
        assert!(manifest.declares(0b0010));
        assert!(manifest.is_trusted_for_execution());
    }

    #[test]
    fn lifecycle_state_mapping_is_deterministic() {
        assert_eq!(lifecycle_maps_to_state(DriverLifecycle::Initialize), DriverState::Starting);
        assert_eq!(lifecycle_maps_to_state(DriverLifecycle::Run), DriverState::Running);
        assert_eq!(lifecycle_maps_to_state(DriverLifecycle::Stop), DriverState::Stopping);
        assert_eq!(lifecycle_maps_to_state(DriverLifecycle::Remove), DriverState::Quarantined);
    }
}

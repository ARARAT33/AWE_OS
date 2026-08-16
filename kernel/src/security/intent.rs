#![no_std]

use super::capability::{Capability, Rights};

/// Why a privileged action is being requested.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct IntentCode(pub u32);

pub mod intent {
    use super::IntentCode;
    pub const READ: IntentCode = IntentCode(1);
    pub const WRITE: IntentCode = IntentCode(2);
    pub const EXECUTE: IntentCode = IntentCode(3);
    pub const DEVICE_CONTROL: IntentCode = IntentCode(4);
    pub const ADMIN: IntentCode = IntentCode(5);
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Intent {
    pub code: IntentCode,
    pub required: Rights,
    pub impact: Impact,
    pub expected_resource_units: u64,
}

impl Intent {
    pub const fn new(code: IntentCode, required: Rights, impact: Impact, budget: u64) -> Self {
        Self { code, required, impact, expected_resource_units: budget }
    }

    pub const fn is_consistent(&self) -> bool {
        self.expected_resource_units != 0 && self.required.0 != 0
    }

    pub const fn authorize(&self, capability: Capability) -> bool {
        self.is_consistent() && capability.permits(self.required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::intent::*;
    use crate::security::capability::CapabilityId;

    #[test]
    fn intent_requires_matching_capability() {
        let cap = Capability { id: CapabilityId(7), rights: Rights::READ };
        let request = Intent::new(READ, Rights::READ, Impact::Low, 1);
        assert!(request.authorize(cap));
        let write = Intent::new(WRITE, Rights::WRITE, Impact::Medium, 1);
        assert!(!write.authorize(cap));
    }

    #[test]
    fn malformed_intent_is_rejected() {
        let cap = Capability { id: CapabilityId(1), rights: Rights::ADMIN };
        let malformed = Intent::new(ADMIN, Rights::NONE, Impact::Critical, 0);
        assert!(!malformed.authorize(cap));
    }
}

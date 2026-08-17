#![no_std]

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CapabilityId(pub u64);

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Rights(pub u64);

impl Rights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const MAP: Self = Self(1 << 3);
    pub const DEVICE: Self = Self(1 << 4);
    pub const NETWORK: Self = Self(1 << 5);
    pub const ADMIN: Self = Self(1 << 63);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Capability {
    pub id: CapabilityId,
    pub rights: Rights,
}

impl Capability {
    pub const fn permits(&self, required: Rights) -> bool {
        self.rights.contains(required)
    }

    /// Derive a strictly weaker capability. Escalation is impossible because
    /// the resulting rights are an intersection with the parent capability.
    pub const fn derive(&self, id: CapabilityId, requested: Rights) -> Self {
        Self {
            id,
            rights: self.rights.intersect(requested),
        }
    }

    /// Revocation is represented by deriving an empty capability. Callers must
    /// retain the returned value; no global mutable authority is introduced.
    pub const fn revoked(id: CapabilityId) -> Self {
        Self {
            id,
            rights: Rights::NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivation_cannot_escalate() {
        let root = Capability {
            id: CapabilityId(1),
            rights: Rights::READ.union(Rights::WRITE),
        };
        let child = root.derive(CapabilityId(2), Rights::WRITE.union(Rights::NETWORK));
        assert!(child.permits(Rights::WRITE));
        assert!(!child.permits(Rights::NETWORK));
        assert!(!child.permits(Rights::READ));
    }

    #[test]
    fn revoked_capability_has_no_authority() {
        let cap = Capability::revoked(CapabilityId(3));
        assert!(cap.rights.is_empty());
        assert!(!cap.permits(Rights::READ));
    }
}

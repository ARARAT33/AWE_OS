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

    pub const fn contains(self, other: Self) -> bool { (self.0 & other.0) == other.0 }
    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Capability {
    pub id: CapabilityId,
    pub rights: Rights,
}

impl Capability {
    pub const fn permits(&self, required: Rights) -> bool { self.rights.contains(required) }
}

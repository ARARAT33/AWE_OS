//! Native AWEOS runtime kernel contract.
//! Runtime handles are capability-scoped and all privileged operations remain
//! behind explicit validation boundaries.

#![allow(dead_code)]

mod end_user;
pub mod graphics;
pub mod system;

pub use end_user::{
    AppRecord, AppState, EndUserRuntime, EndUserRuntimeError, FramebufferInfo, InputEvent,
    RuntimeEvent, ServiceRecord, ServiceState,
};
pub use graphics::{DoubleBuffer, Rect as RuntimeRect, Window, WindowError, WindowManager};
pub use system::SystemRuntime;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilitySet(pub u64);
impl CapabilitySet {
    pub const NONE: Self = Self(0);
    pub const PROCESS: Self = Self(1 << 0);
    pub const MEMORY: Self = Self(1 << 1);
    pub const IPC: Self = Self(1 << 2);
    pub const DEVICE: Self = Self(1 << 3);
    pub const STORAGE: Self = Self(1 << 4);
    pub const NETWORK: Self = Self(1 << 5);
    pub const UI: Self = Self(1 << 6);
    pub const fn contains(self, required: Self) -> bool { (self.0 & required.0) == required.0 }
    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeError { CapabilityDenied, InvalidHandle, ResourceExhausted }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeContext { pub capabilities: CapabilitySet }
impl RuntimeContext {
    pub const fn new(capabilities: CapabilitySet) -> Self { Self { capabilities } }
    pub const fn require(self, required: CapabilitySet) -> Result<(), RuntimeError> {
        if self.capabilities.contains(required) { Ok(()) } else { Err(RuntimeError::CapabilityDenied) }
    }
}

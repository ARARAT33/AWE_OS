#![no_std]

//! AWE_OS update/recovery contract. The service owns lifecycle policy for
//! atomic A/B updates; CellKernel only exposes bounded activation primitives.

pub const UPDATE_ABI_MAJOR: u16 = 1;
pub const UPDATE_ABI_MINOR: u16 = 2;
pub const MAX_VERSION_LEN: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot { A, B }
impl Slot { pub const fn other(self) -> Self { match self { Self::A => Self::B, Self::B => Self::A } } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState { Empty, Staged, Booting, Healthy, Failed, Quarantined }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version { pub bytes: [u8; MAX_VERSION_LEN], pub len: u8 }
impl Version {
    pub const fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > MAX_VERSION_LEN { return None; }
        let mut out = [0u8; MAX_VERSION_LEN]; let mut i = 0;
        while i < bytes.len() { out[i] = bytes[i]; i += 1; }
        Some(Self { bytes: out, len: bytes.len() as u8 })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateManifest {
    pub version: Version,
    pub generation: u64,
    pub payload_len: u64,
    pub payload_digest: [u8; 32],
    pub min_generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateError { InvalidManifest, Downgrade, PayloadTooLarge, InvalidTransition, Quarantined, NotStaged }
pub const MAX_PAYLOAD: u64 = 4 * 1024 * 1024 * 1024;

pub const fn validate_manifest(m: UpdateManifest, current_generation: u64) -> Result<(), UpdateError> {
    if m.version.len == 0 || m.version.len as usize > MAX_VERSION_LEN || m.payload_len == 0 { return Err(UpdateError::InvalidManifest); }
    if m.payload_len > MAX_PAYLOAD { return Err(UpdateError::PayloadTooLarge); }
    if m.generation < current_generation || m.min_generation > m.generation { return Err(UpdateError::Downgrade); }
    Ok(())
}

const fn transition(from: SlotState, to: SlotState) -> bool {
    matches!((from, to),
        (SlotState::Empty, SlotState::Staged) |
        (SlotState::Staged, SlotState::Booting) |
        (SlotState::Booting, SlotState::Healthy) |
        (SlotState::Booting, SlotState::Failed) |
        (SlotState::Failed, SlotState::Staged) |
        (SlotState::Healthy, SlotState::Staged) |
        (SlotState::Failed, SlotState::Quarantined))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateManager {
    active: Slot,
    generation: u64,
    states: [SlotState; 2],
    manifests: [Option<UpdateManifest>; 2],
}

impl UpdateManager {
    pub const fn new(generation: u64) -> Self { Self { active: Slot::A, generation, states: [SlotState::Healthy, SlotState::Empty], manifests: [None, None] } }
    const fn index(slot: Slot) -> usize { match slot { Slot::A => 0, Slot::B => 1 } }
    pub const fn active(&self) -> Slot { self.active }
    pub const fn generation(&self) -> u64 { self.generation }
    pub const fn state(&self, slot: Slot) -> SlotState { self.states[Self::index(slot)] }

    pub fn stage(&mut self, slot: Slot, manifest: UpdateManifest) -> Result<(), UpdateError> {
        validate_manifest(manifest, self.generation)?;
        let i = Self::index(slot);
        if self.states[i] == SlotState::Quarantined { return Err(UpdateError::Quarantined); }
        if !transition(self.states[i], SlotState::Staged) { return Err(UpdateError::InvalidTransition); }
        self.manifests[i] = Some(manifest); self.states[i] = SlotState::Staged; Ok(())
    }

    pub fn boot_pending(&mut self) -> Result<Slot, UpdateError> {
        let slot = self.active.other(); let i = Self::index(slot);
        if self.states[i] != SlotState::Staged || self.manifests[i].is_none() { return Err(UpdateError::NotStaged); }
        self.states[i] = SlotState::Booting; Ok(slot)
    }

    pub fn mark_healthy(&mut self, slot: Slot) -> Result<(), UpdateError> {
        let i = Self::index(slot); if self.states[i] != SlotState::Booting { return Err(UpdateError::InvalidTransition); }
        let manifest = match self.manifests[i] { Some(m) => m, None => return Err(UpdateError::InvalidManifest) };
        if manifest.generation < self.generation { return Err(UpdateError::Downgrade); }
        self.active = slot; self.generation = manifest.generation; self.states[i] = SlotState::Healthy;
        let old = Self::index(slot.other());
        if self.states[old] == SlotState::Healthy { self.states[old] = SlotState::Staged; }
        Ok(())
    }

    pub fn mark_failed(&mut self, slot: Slot) -> Result<(), UpdateError> {
        let i = Self::index(slot); if self.states[i] != SlotState::Booting { return Err(UpdateError::InvalidTransition); }
        self.states[i] = SlotState::Failed; Ok(())
    }

    /// Recover a booting/failed target without changing the trusted generation.
    pub fn recover_failed(&mut self, slot: Slot) -> Result<(), UpdateError> {
        let i = Self::index(slot);
        if self.states[i] != SlotState::Failed && self.states[i] != SlotState::Booting { return Err(UpdateError::InvalidTransition); }
        self.states[i] = SlotState::Failed;
        self.active = slot.other();
        Ok(())
    }

    pub fn rollback(&mut self, slot: Slot) -> Result<(), UpdateError> { self.recover_failed(slot) }
}

pub mod transaction;
pub use transaction::Transaction;

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest(generation: u64) -> UpdateManifest { UpdateManifest { version: Version::new(b"1.1.0").unwrap(), generation, payload_len: 1024, payload_digest: [7; 32], min_generation: generation } }
    #[test] fn atomic_ab_update_promotes_only_healthy_slot() { let mut m = UpdateManager::new(1); m.stage(Slot::B, manifest(2)).unwrap(); assert_eq!(m.boot_pending().unwrap(), Slot::B); m.mark_healthy(Slot::B).unwrap(); assert_eq!(m.active(), Slot::B); assert_eq!(m.generation(), 2); }
    #[test] fn failed_update_rolls_back() { let mut m = UpdateManager::new(1); m.stage(Slot::B, manifest(2)).unwrap(); m.boot_pending().unwrap(); m.mark_failed(Slot::B).unwrap(); m.rollback(Slot::B).unwrap(); assert_eq!(m.active(), Slot::A); }
    #[test] fn downgrades_are_rejected() { assert_eq!(validate_manifest(manifest(1), 2), Err(UpdateError::Downgrade)); }
    #[test] fn transaction_recovery_is_fail_closed() { let mut m = UpdateManager::new(4); let tx = Transaction::begin(&m, Slot::B, manifest(5)).unwrap(); m.stage(Slot::B, manifest(5)).unwrap(); m.boot_pending().unwrap(); tx.recover(&mut m).unwrap(); assert_eq!(m.active(), Slot::A); assert_eq!(m.state(Slot::B), SlotState::Failed); }
}

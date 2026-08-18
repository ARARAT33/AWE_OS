#![no_std]

//! AWE_OS update/recovery contract. The service owns lifecycle policy for
//! atomic A/B updates; CellKernel only exposes the bounded primitives needed
//! to activate a selected slot.

pub const UPDATE_ABI_MAJOR: u16 = 1;
pub const UPDATE_ABI_MINOR: u16 = 1;
pub const MAX_VERSION_LEN: usize = 32;
pub const MAX_REASON_LEN: usize = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    A,
    B,
}

impl Slot {
    pub const fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Staged,
    Booting,
    Healthy,
    Failed,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Version {
    pub bytes: [u8; MAX_VERSION_LEN],
    pub len: u8,
}

impl Version {
    pub const fn new(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > MAX_VERSION_LEN {
            return None;
        }
        let mut out = [0u8; MAX_VERSION_LEN];
        let mut i = 0;
        while i < bytes.len() {
            out[i] = bytes[i];
            i += 1;
        }
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
pub enum UpdateError {
    InvalidManifest,
    Downgrade,
    PayloadTooLarge,
    InvalidTransition,
    Quarantined,
    NotStaged,
}

pub const MAX_PAYLOAD: u64 = 4 * 1024 * 1024 * 1024;

pub const fn validate_manifest(m: UpdateManifest, current_generation: u64) -> Result<(), UpdateError> {
    if m.version.len == 0 || m.version.len as usize > MAX_VERSION_LEN || m.payload_len == 0 {
        return Err(UpdateError::InvalidManifest);
    }
    if m.payload_len > MAX_PAYLOAD {
        return Err(UpdateError::PayloadTooLarge);
    }
    if m.generation < current_generation || m.min_generation > m.generation {
        return Err(UpdateError::Downgrade);
    }
    Ok(())
}

const fn transition(from: SlotState, to: SlotState) -> bool {
    matches!((from, to),
        (SlotState::Empty, SlotState::Staged)
            | (SlotState::Staged, SlotState::Booting)
            | (SlotState::Booting, SlotState::Healthy)
            | (SlotState::Booting, SlotState::Failed)
            | (SlotState::Failed, SlotState::Staged)
            | (SlotState::Healthy, SlotState::Staged)
            | (SlotState::Failed, SlotState::Quarantined))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpdateManager {
    active: Slot,
    generation: u64,
    states: [SlotState; 2],
    manifests: [Option<UpdateManifest>; 2],
}

impl UpdateManager {
    pub const fn new(generation: u64) -> Self {
        Self {
            active: Slot::A,
            generation,
            states: [SlotState::Healthy, SlotState::Empty],
            manifests: [None, None],
        }
    }

    const fn index(slot: Slot) -> usize {
        match slot { Slot::A => 0, Slot::B => 1 }
    }

    pub const fn active(&self) -> Slot { self.active }
    pub const fn generation(&self) -> u64 { self.generation }
    pub const fn state(&self, slot: Slot) -> SlotState { self.states[Self::index(slot)] }

    pub fn stage(&mut self, slot: Slot, manifest: UpdateManifest) -> Result<(), UpdateError> {
        validate_manifest(manifest, self.generation)?;
        let i = Self::index(slot);
        if self.states[i] == SlotState::Quarantined || !transition(self.states[i], SlotState::Staged) {
            return Err(UpdateError::InvalidTransition);
        }
        self.manifests[i] = Some(manifest);
        self.states[i] = SlotState::Staged;
        Ok(())
    }

    pub fn boot_pending(&mut self) -> Result<Slot, UpdateError> {
        let slot = self.active.other();
        let i = Self::index(slot);
        if self.states[i] != SlotState::Staged || self.manifests[i].is_none() {
            return Err(UpdateError::NotStaged);
        }
        self.states[i] = SlotState::Booting;
        Ok(slot)
    }

    pub fn mark_healthy(&mut self, slot: Slot) -> Result<(), UpdateError> {
        let i = Self::index(slot);
        if self.states[i] != SlotState::Booting || self.manifests[i].is_none() {
            return Err(UpdateError::InvalidTransition);
        }
        let manifest = self.manifests[i].unwrap();
        self.active = slot;
        self.generation = manifest.generation;
        self.states[i] = SlotState::Healthy;
        self.states[Self::index(slot.other())] = SlotState::Staged;
        Ok(())
    }

    pub fn mark_failed(&mut self, slot: Slot) -> Result<(), UpdateError> {
        let i = Self::index(slot);
        if self.states[i] != SlotState::Booting {
            return Err(UpdateError::InvalidTransition);
        }
        self.states[i] = SlotState::Failed;
        Ok(())
    }

    pub fn rollback(&mut self, slot: Slot) -> Result<(), UpdateError> {
        let i = Self::index(slot);
        if self.states[i] != SlotState::Failed && self.states[i] != SlotState::Booting {
            return Err(UpdateError::InvalidTransition);
        }
        self.states[i] = SlotState::Failed;
        self.active = slot.other();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(generation: u64) -> UpdateManifest {
        UpdateManifest {
            version: Version::new(b"1.1.0").unwrap(),
            generation,
            payload_len: 1024,
            payload_digest: [7; 32],
            min_generation: generation,
        }
    }

    #[test]
    fn atomic_ab_update_promotes_only_healthy_slot() {
        let mut manager = UpdateManager::new(1);
        manager.stage(Slot::B, manifest(2)).unwrap();
        assert_eq!(manager.boot_pending().unwrap(), Slot::B);
        manager.mark_healthy(Slot::B).unwrap();
        assert_eq!(manager.active(), Slot::B);
        assert_eq!(manager.generation(), 2);
    }

    #[test]
    fn failed_update_rolls_back() {
        let mut manager = UpdateManager::new(1);
        manager.stage(Slot::B, manifest(2)).unwrap();
        manager.boot_pending().unwrap();
        manager.mark_failed(Slot::B).unwrap();
        manager.rollback(Slot::B).unwrap();
        assert_eq!(manager.active(), Slot::A);
    }

    #[test]
    fn downgrades_are_rejected() {
        assert_eq!(validate_manifest(manifest(1), 2), Err(UpdateError::Downgrade));
    }
}

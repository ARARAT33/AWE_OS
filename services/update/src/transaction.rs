#![no_std]

use crate::{Slot, SlotState, UpdateError, UpdateManager, UpdateManifest};

/// Crash-safe update transaction metadata. The state is intentionally small so
/// a persistent backend can journal it atomically before changing boot state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub target: Slot,
    pub previous: Slot,
    pub generation: u64,
    pub committed: bool,
}

impl Transaction {
    pub const fn begin(manager: &UpdateManager, target: Slot, manifest: UpdateManifest) -> Result<Self, UpdateError> {
        if target == manager.active() {
            return Err(UpdateError::InvalidTransition);
        }
        if manifest.generation < manager.generation() {
            return Err(UpdateError::Downgrade);
        }
        Ok(Self {
            target,
            previous: manager.active(),
            generation: manifest.generation,
            committed: false,
        })
    }

    pub const fn commit(self) -> Self {
        Self { committed: true, ..self }
    }

    pub const fn recover(self, manager: &mut UpdateManager) -> Result<(), UpdateError> {
        if self.committed {
            return Ok(());
        }
        if manager.state(self.target) == SlotState::Booting {
            manager.recover_failed(self.target)
        } else {
            Ok(())
        }
    }
}

use crate::{
    DriverClass, DriverId, DriverManifest, DriverState, DriverTrust, MAX_REGISTERED_DRIVERS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverDescriptor {
    pub id: DriverId,
    pub class: DriverClass,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub vendor: u16,
    pub device: u16,
    pub state: DriverState,
    pub trust: DriverTrust,
}

impl DriverDescriptor {
    pub const fn manifest(self, architecture_mask: u64, capability_mask: u64) -> DriverManifest {
        DriverManifest::new(
            self.id,
            self.class,
            self.abi_major,
            self.abi_minor,
            architecture_mask,
            capability_mask,
            self.trust,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Full,
    Duplicate,
    NotFound,
    Untrusted,
}

pub struct DriverRegistry {
    entries: [Option<DriverDescriptor>; MAX_REGISTERED_DRIVERS],
    len: usize,
}

impl DriverRegistry {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_REGISTERED_DRIVERS],
            len: 0,
        }
    }
    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn register(&mut self, descriptor: DriverDescriptor) -> Result<(), RegistryError> {
        if self.find(descriptor.id).is_some() {
            return Err(RegistryError::Duplicate);
        }
        if !matches!(descriptor.trust, DriverTrust::Verified) {
            return Err(RegistryError::Untrusted);
        }
        if self.len == MAX_REGISTERED_DRIVERS {
            return Err(RegistryError::Full);
        }
        self.entries[self.len] = Some(descriptor);
        self.len += 1;
        Ok(())
    }

    pub fn find(&self, id: DriverId) -> Option<&DriverDescriptor> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &self.entries[i] {
                if entry.id == id {
                    return Some(entry);
                }
            }
            i += 1;
        }
        None
    }

    pub fn set_state(&mut self, id: DriverId, state: DriverState) -> Result<(), RegistryError> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &mut self.entries[i] {
                if entry.id == id {
                    entry.state = state;
                    return Ok(());
                }
            }
            i += 1;
        }
        Err(RegistryError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(id: DriverId, trust: DriverTrust) -> DriverDescriptor {
        DriverDescriptor {
            id,
            class: DriverClass::Virtio,
            abi_major: 1,
            abi_minor: 2,
            vendor: 0x1af4,
            device: 0x1001,
            state: DriverState::Discovered,
            trust,
        }
    }

    #[test]
    fn registry_is_bounded_and_tracks_state() {
        let mut r = DriverRegistry::new();
        let id = DriverId(1);
        assert!(r.register(descriptor(id, DriverTrust::Verified)).is_ok());
        assert_eq!(r.set_state(id, DriverState::Running), Ok(()));
        assert_eq!(r.find(id).unwrap().state, DriverState::Running);
    }

    #[test]
    fn registry_rejects_untrusted_descriptors() {
        let mut r = DriverRegistry::new();
        assert_eq!(
            r.register(descriptor(DriverId(2), DriverTrust::Unverified)),
            Err(RegistryError::Untrusted)
        );
    }

    #[test]
    fn descriptor_exports_manifest_contract() {
        let d = descriptor(DriverId(3), DriverTrust::Verified);
        let m = d.manifest(0b10, 0b100);
        assert_eq!(m.id, d.id);
        assert_eq!(m.abi_minor, 2);
        assert!(m.is_trusted_for_execution());
    }
}

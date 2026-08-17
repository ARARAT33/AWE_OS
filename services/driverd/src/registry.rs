use crate::{DriverClass, DriverId, DriverState, MAX_REGISTERED_DRIVERS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverDescriptor {
    pub id: DriverId,
    pub class: DriverClass,
    pub abi_major: u16,
    pub vendor: u16,
    pub device: u16,
    pub state: DriverState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Full,
    Duplicate,
    NotFound,
}

pub struct DriverRegistry {
    entries: [Option<DriverDescriptor>; MAX_REGISTERED_DRIVERS],
    len: usize,
}

impl DriverRegistry {
    pub const fn new() -> Self { Self { entries: [None; MAX_REGISTERED_DRIVERS], len: 0 } }
    pub const fn len(&self) -> usize { self.len }

    pub fn register(&mut self, descriptor: DriverDescriptor) -> Result<(), RegistryError> {
        if self.find(descriptor.id).is_some() { return Err(RegistryError::Duplicate); }
        if self.len == MAX_REGISTERED_DRIVERS { return Err(RegistryError::Full); }
        self.entries[self.len] = Some(descriptor);
        self.len += 1;
        Ok(())
    }

    pub fn find(&self, id: DriverId) -> Option<&DriverDescriptor> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &self.entries[i] { if entry.id == id { return Some(entry); } }
            i += 1;
        }
        None
    }

    pub fn set_state(&mut self, id: DriverId, state: DriverState) -> Result<(), RegistryError> {
        let mut i = 0;
        while i < self.len {
            if let Some(entry) = &mut self.entries[i] {
                if entry.id == id { entry.state = state; return Ok(()); }
            }
            i += 1;
        }
        Err(RegistryError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_is_bounded() {
        let mut r = DriverRegistry::new();
        let id = DriverId(1);
        assert!(r.register(DriverDescriptor { id, class: DriverClass::Virtio, abi_major: 1, vendor: 0x1af4, device: 0x1001, state: DriverState::Discovered }).is_ok());
        assert_eq!(r.set_state(id, DriverState::Running), Ok(()));
        assert_eq!(r.find(id).unwrap().state, DriverState::Running);
    }
}

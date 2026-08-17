#![no_std]

mod access;
mod binding;

pub use access::{
    AccessKind, DeviceAccessContract, InterruptGrant, IoRegion, IrqMode, PowerState,
    PowerTransitionError, power_transition,
};
pub use binding::{BindingDecision, DeviceMatch, MatchKind, ResourceGrant, decide_binding};

/// Stable device identity used by the AWE driver registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceClass {
    Unknown,
    Storage,
    Network,
    Input,
    Display,
    Audio,
    Console,
    Timer,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceState {
    Discovered,
    Bound,
    Active,
    Failed,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceContract {
    pub id: DeviceId,
    pub class: DeviceClass,
    pub state: DeviceState,
    pub vendor: u16,
    pub product: u16,
    pub irq: Option<u32>,
}

impl DeviceContract {
    pub const fn new(id: DeviceId, class: DeviceClass, vendor: u16, product: u16) -> Self {
        Self {
            id,
            class,
            state: DeviceState::Discovered,
            vendor,
            product,
            irq: None,
        }
    }

    pub const fn with_irq(mut self, irq: u32) -> Self {
        self.irq = Some(irq);
        self
    }

    pub const fn matching(self) -> DeviceMatch {
        DeviceMatch::exact(self.vendor, self.product, self.class)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Full,
    Duplicate,
    NotFound,
}

/// Fixed-capacity device registry. Discovery is bounded and deterministic.
pub struct DeviceRegistry<const N: usize> {
    devices: [Option<DeviceContract>; N],
    len: usize,
}

impl<const N: usize> DeviceRegistry<N> {
    pub const fn new() -> Self {
        Self {
            devices: [None; N],
            len: 0,
        }
    }
    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn register(&mut self, device: DeviceContract) -> Result<(), RegistryError> {
        if self.find(device.id).is_some() {
            return Err(RegistryError::Duplicate);
        }
        if self.len == N {
            return Err(RegistryError::Full);
        }
        self.devices[self.len] = Some(device);
        self.len += 1;
        Ok(())
    }

    pub fn find(&self, id: DeviceId) -> Option<&DeviceContract> {
        let mut i = 0;
        while i < self.len {
            if let Some(device) = &self.devices[i] {
                if device.id == id {
                    return Some(device);
                }
            }
            i += 1;
        }
        None
    }

    pub fn set_state(&mut self, id: DeviceId, state: DeviceState) -> Result<(), RegistryError> {
        let mut i = 0;
        while i < self.len {
            if let Some(device) = &mut self.devices[i] {
                if device.id == id {
                    device.state = state;
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

    #[test]
    fn registry_is_bounded_and_tracks_state() {
        let mut registry: DeviceRegistry<1> = DeviceRegistry::new();
        let id = DeviceId(7);
        assert!(
            registry
                .register(DeviceContract::new(id, DeviceClass::Storage, 1, 2))
                .is_ok()
        );
        assert_eq!(
            registry.register(DeviceContract::new(id, DeviceClass::Storage, 1, 2)),
            Err(RegistryError::Duplicate)
        );
        assert_eq!(registry.set_state(id, DeviceState::Active), Ok(()));
        assert_eq!(registry.find(id).unwrap().state, DeviceState::Active);
        assert_eq!(
            registry.register(DeviceContract::new(DeviceId(8), DeviceClass::Network, 3, 4)),
            Err(RegistryError::Full)
        );
    }

    #[test]
    fn device_contract_exposes_canonical_exact_match() {
        let device = DeviceContract::new(DeviceId(11), DeviceClass::Display, 0x10de, 0x1cb3);
        assert_eq!(
            device.matching(),
            DeviceMatch::exact(0x10de, 0x1cb3, DeviceClass::Display)
        );
    }
}

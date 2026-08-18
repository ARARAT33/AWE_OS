//! Bounded native-driver lifecycle state machine.
//! The policy is transport-neutral and deliberately does not perform hardware I/O.

use super::{DeviceId, DeviceState, RegistryError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    NotFound,
    InvalidTransition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleEvent {
    Bind,
    Initialize,
    Start,
    Suspend,
    Resume,
    Stop,
    Remove,
    Fault,
    Recover,
}

pub const MAX_DRIVER_RESTARTS: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DriverLifecycle {
    pub device: DeviceId,
    pub state: DeviceState,
    pub restarts: u8,
}

impl DriverLifecycle {
    pub const fn new(device: DeviceId) -> Self {
        Self { device, state: DeviceState::Discovered, restarts: 0 }
    }

    pub fn apply(&mut self, event: LifecycleEvent) -> Result<DeviceState, LifecycleError> {
        let next = match (self.state, event) {
            (DeviceState::Discovered, LifecycleEvent::Bind) => DeviceState::Bound,
            (DeviceState::Bound, LifecycleEvent::Initialize) => DeviceState::Active,
            (DeviceState::Active, LifecycleEvent::Suspend) => DeviceState::Bound,
            (DeviceState::Bound, LifecycleEvent::Resume) => DeviceState::Active,
            (DeviceState::Active, LifecycleEvent::Stop) => DeviceState::Bound,
            (DeviceState::Bound, LifecycleEvent::Remove) => DeviceState::Discovered,
            (DeviceState::Active, LifecycleEvent::Fault) => DeviceState::Failed,
            (DeviceState::Failed, LifecycleEvent::Recover) if self.restarts < MAX_DRIVER_RESTARTS => {
                self.restarts += 1;
                DeviceState::Bound
            }
            (DeviceState::Failed, LifecycleEvent::Recover) => DeviceState::Quarantined,
            (DeviceState::Quarantined, LifecycleEvent::Remove) => DeviceState::Discovered,
            _ => return Err(LifecycleError::InvalidTransition),
        };
        self.state = next;
        Ok(next)
    }
}

pub fn sync_registry<const N: usize>(
    registry: &mut super::DeviceRegistry<N>,
    lifecycle: DriverLifecycle,
) -> Result<(), LifecycleError> {
    registry
        .set_state(lifecycle.device, lifecycle.state)
        .map_err(|error| match error {
            RegistryError::NotFound => LifecycleError::NotFound,
            RegistryError::Full | RegistryError::Duplicate => LifecycleError::NotFound,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::{DeviceClass, DeviceContract, DeviceRegistry};

    #[test]
    fn lifecycle_rejects_invalid_order_and_recovers_boundedly() {
        let mut driver = DriverLifecycle::new(DeviceId(1));
        assert_eq!(driver.apply(LifecycleEvent::Start), Err(LifecycleError::InvalidTransition));
        driver.apply(LifecycleEvent::Bind).unwrap();
        driver.apply(LifecycleEvent::Initialize).unwrap();
        driver.apply(LifecycleEvent::Fault).unwrap();
        for _ in 0..MAX_DRIVER_RESTARTS { driver.apply(LifecycleEvent::Recover).unwrap(); driver.apply(LifecycleEvent::Initialize).unwrap(); driver.apply(LifecycleEvent::Fault).unwrap(); }
        assert_eq!(driver.apply(LifecycleEvent::Recover), Ok(DeviceState::Quarantined));
    }

    #[test]
    fn lifecycle_state_can_be_published_to_registry() {
        let mut registry: DeviceRegistry<1> = DeviceRegistry::new();
        registry.register(DeviceContract::new(DeviceId(9), DeviceClass::Network, 1, 2)).unwrap();
        let mut driver = DriverLifecycle::new(DeviceId(9));
        driver.apply(LifecycleEvent::Bind).unwrap();
        sync_registry(&mut registry, driver).unwrap();
        assert_eq!(registry.find(DeviceId(9)).unwrap().state, DeviceState::Bound);
    }
}

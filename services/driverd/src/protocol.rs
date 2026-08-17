#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverClass {
    Storage,
    Network,
    Display,
    Input,
    Audio,
    Virtio,
    Compatibility,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverState {
    Discovered,
    Starting,
    Running,
    Stopping,
    Failed,
    Quarantined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverCommand {
    Probe,
    Start,
    Stop,
    Reset,
    Quarantine,
    HealthCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverReply {
    Accepted,
    Busy,
    Unsupported,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverEvent {
    Registered(DriverId),
    StateChanged(DriverId, DriverState),
    Fault(DriverId),
}

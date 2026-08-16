#![no_std]

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BootPhase {
    LoaderHandoff = 0,
    ProtocolValidated = 1,
    MemoryReady = 2,
    InterruptsReady = 3,
    SchedulerReady = 4,
    UserspaceReady = 5,
    Running = 6,
}

impl BootPhase {
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::LoaderHandoff => Some(Self::ProtocolValidated),
            Self::ProtocolValidated => Some(Self::MemoryReady),
            Self::MemoryReady => Some(Self::InterruptsReady),
            Self::InterruptsReady => Some(Self::SchedulerReady),
            Self::SchedulerReady => Some(Self::UserspaceReady),
            Self::UserspaceReady => Some(Self::Running),
            Self::Running => None,
        }
    }

    pub const fn is_running(self) -> bool { matches!(self, Self::Running) }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BootFailure {
    InvalidTransition = 1,
    RequiredSubsystemUnavailable = 2,
}

pub struct BootProgress {
    phase: BootPhase,
    failure: Option<BootFailure>,
}

impl BootProgress {
    pub const fn new() -> Self {
        Self { phase: BootPhase::LoaderHandoff, failure: None }
    }

    pub const fn phase(&self) -> BootPhase { self.phase }
    pub const fn failure(&self) -> Option<BootFailure> { self.failure }
    pub const fn is_failed(&self) -> bool { self.failure.is_some() }

    /// Boot can only advance one validated phase at a time. Once a failure is
    /// recorded, later code cannot accidentally promote the system to Running.
    pub fn advance(&mut self) -> bool {
        if self.failure.is_some() { return false; }
        match self.phase.next() {
            Some(next) => { self.phase = next; true }
            None => false,
        }
    }

    pub fn fail(&mut self, reason: BootFailure) {
        if self.failure.is_none() { self.failure = Some(reason); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_progress_is_monotonic() {
        let mut progress = BootProgress::new();
        for _ in 0..6 { assert!(progress.advance()); }
        assert!(progress.phase().is_running());
        assert!(!progress.advance());
    }

    #[test]
    fn failure_is_terminal() {
        let mut progress = BootProgress::new();
        progress.fail(BootFailure::RequiredSubsystemUnavailable);
        assert!(progress.is_failed());
        assert_eq!(progress.failure(), Some(BootFailure::RequiredSubsystemUnavailable));
        assert!(!progress.advance());
        assert!(!progress.phase().is_running());
    }
}

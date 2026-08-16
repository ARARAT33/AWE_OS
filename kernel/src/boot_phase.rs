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

    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running)
    }
}

pub struct BootProgress {
    phase: BootPhase,
}

impl BootProgress {
    pub const fn new() -> Self {
        Self { phase: BootPhase::LoaderHandoff }
    }

    pub const fn phase(&self) -> BootPhase {
        self.phase
    }

    pub fn advance(&mut self) -> bool {
        match self.phase.next() {
            Some(next) => {
                self.phase = next;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_progress_is_monotonic() {
        let mut progress = BootProgress::new();
        for _ in 0..6 {
            assert!(progress.advance());
        }
        assert!(progress.phase().is_running());
        assert!(!progress.advance());
    }
}

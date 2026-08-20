#![no_std]

use super::DeviceId;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Unknown = 0,
    Success = 1,
    Failure = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DriverExperience {
    pub device: DeviceId,
    pub driver_id: u32,
    pub attempts: u32,
    pub successes: u32,
    pub failures: u32,
    pub last: ProbeOutcome,
}

impl DriverExperience {
    pub const fn new(device: DeviceId, driver_id: u32) -> Self {
        Self {
            device,
            driver_id,
            attempts: 0,
            successes: 0,
            failures: 0,
            last: ProbeOutcome::Unknown,
        }
    }
    pub fn record(&mut self, outcome: ProbeOutcome) {
        self.attempts = self.attempts.saturating_add(1);
        match outcome {
            ProbeOutcome::Success => self.successes = self.successes.saturating_add(1),
            ProbeOutcome::Failure => self.failures = self.failures.saturating_add(1),
            ProbeOutcome::Unknown => {}
        }
        self.last = outcome;
    }
    pub const fn stable(&self) -> bool {
        self.attempts >= 3 && self.successes > self.failures
    }
}

pub struct ExperienceDb<const N: usize> {
    entries: [Option<DriverExperience>; N],
}

impl<const N: usize> ExperienceDb<N> {
    pub const fn new() -> Self {
        Self { entries: [None; N] }
    }
    pub fn record(&mut self, device: DeviceId, driver_id: u32, outcome: ProbeOutcome) -> bool {
        let mut i = 0;
        while i < N {
            if let Some(ref mut e) = self.entries[i] {
                if e.device == device && e.driver_id == driver_id {
                    e.record(outcome);
                    return true;
                }
            }
            i += 1;
        }
        let mut j = 0;
        while j < N {
            if self.entries[j].is_none() {
                let mut e = DriverExperience::new(device, driver_id);
                e.record(outcome);
                self.entries[j] = Some(e);
                return true;
            }
            j += 1;
        }
        false
    }
    pub fn get(&self, device: DeviceId, driver_id: u32) -> Option<DriverExperience> {
        let mut i = 0;
        while i < N {
            if let Some(e) = self.entries[i] {
                if e.device == device && e.driver_id == driver_id {
                    return Some(e);
                }
            }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn experience_is_bounded_and_deterministic() {
        let mut db: ExperienceDb<2> = ExperienceDb::new();
        let id = DeviceId::new(1, 2);
        assert!(db.record(id, 7, ProbeOutcome::Success));
        assert!(db.record(id, 7, ProbeOutcome::Success));
        assert!(db.record(id, 7, ProbeOutcome::Success));
        assert!(db.get(id, 7).unwrap().stable());
    }
}

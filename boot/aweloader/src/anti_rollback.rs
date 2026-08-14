#![no_std]

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub epoch: u64,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32, epoch: u64) -> Self {
        Self { major, minor, patch, epoch }
    }

    pub const fn is_at_least(self, minimum: Self) -> bool {
        if self.epoch != minimum.epoch { return self.epoch > minimum.epoch; }
        if self.major != minimum.major { return self.major > minimum.major; }
        if self.minor != minimum.minor { return self.minor > minimum.minor; }
        self.patch >= minimum.patch
    }
}

pub fn accept(current: Version, candidate: Version) -> bool {
    candidate.is_at_least(current)
}

#![no_std]

use super::bus::DeviceId;
use super::linux_package::LinuxDriverDescriptor;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LinuxCandidate {
    pub descriptor: LinuxDriverDescriptor,
    pub priority: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResolveError { NoMatch, Ambiguous }

/// Deterministic, offline-first Linux driver resolver.
/// The resolver only selects a candidate; installation/execution remains
/// behind the signed package and AWE driver-contract validation layers.
pub fn resolve(device: DeviceId, candidates: &[LinuxCandidate]) -> Result<LinuxCandidate, ResolveError> {
    let mut best: Option<LinuxCandidate> = None;
    let mut ties = 0u8;
    for candidate in candidates {
        let d = candidate.descriptor;
        if d.vendor != device.vendor || d.device != device.device || d.class != device.class || !d.signed || d.module_hash == 0 {
            continue;
        }
        match best {
            None => { best = Some(*candidate); ties = 1; }
            Some(current) if candidate.priority > current.priority => { best = Some(*candidate); ties = 1; }
            Some(current) if candidate.priority == current.priority => {
                if d.api_version > current.descriptor.api_version {
                    best = Some(*candidate);
                    ties = 1;
                } else if d.api_version == current.descriptor.api_version {
                    ties = ties.saturating_add(1);
                }
            }
            _ => {}
        }
    }
    match best {
        None => Err(ResolveError::NoMatch),
        Some(candidate) if ties > 1 => Err(ResolveError::Ambiguous),
        Some(candidate) => Ok(candidate),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DEV: DeviceId = DeviceId { vendor: 0x8086, device: 0x100e, class: 0x0200, revision: 1 };
    fn candidate(priority: u8, api: u32) -> LinuxCandidate {
        LinuxCandidate { descriptor: LinuxDriverDescriptor { vendor: 0x8086, device: 0x100e, class: 0x0200, api_version: api, module_hash: api as u64, signed: true }, priority }
    }
    #[test] fn chooses_highest_priority() { assert_eq!(resolve(DEV, &[candidate(1, 5), candidate(9, 4)]).unwrap().priority, 9); }
    #[test] fn chooses_newer_api_on_priority_tie() { assert_eq!(resolve(DEV, &[candidate(5, 5), candidate(5, 6)]).unwrap().descriptor.api_version, 6); }
    #[test] fn rejects_unsigned_or_wrong_devices() { let mut bad=candidate(9, 6); bad.descriptor.signed=false; assert_eq!(resolve(DEV, &[bad]), Err(ResolveError::NoMatch)); }
}

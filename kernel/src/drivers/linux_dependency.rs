#![no_std]

use super::linux_resolver::LinuxCandidate;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dependency {
    pub driver_hash: u64,
    pub required_hash: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DependencyError {
    Missing,
    Cycle,
}

/// Small, allocation-free dependency validator for the kernel-side install planner.
/// Dependencies are validated before activation; the installer remains transactional.
pub fn validate(candidates: &[LinuxCandidate], deps: &[Dependency]) -> Result<(), DependencyError> {
    for dep in deps {
        let mut found_driver = false;
        let mut found_required = false;
        for c in candidates {
            if c.descriptor.module_hash == dep.driver_hash {
                found_driver = true;
            }
            if c.descriptor.module_hash == dep.required_hash {
                found_required = true;
            }
        }
        if !found_driver || !found_required {
            return Err(DependencyError::Missing);
        }
        if dep.driver_hash == dep.required_hash {
            return Err(DependencyError::Cycle);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::linux_package::LinuxDriverDescriptor;
    use super::*;
    fn c(hash: u64) -> LinuxCandidate {
        LinuxCandidate {
            descriptor: LinuxDriverDescriptor {
                vendor: 1,
                device: 2,
                class: 3,
                api_version: 1,
                module_hash: hash,
                signed: true,
            },
            priority: 1,
        }
    }
    #[test]
    fn validates_dependency() {
        assert!(
            validate(
                &[c(10), c(20)],
                &[Dependency {
                    driver_hash: 10,
                    required_hash: 20
                }]
            )
            .is_ok()
        );
    }
    #[test]
    fn rejects_missing_dependency() {
        assert_eq!(
            validate(
                &[c(10)],
                &[Dependency {
                    driver_hash: 10,
                    required_hash: 20
                }]
            ),
            Err(DependencyError::Missing)
        );
    }
    #[test]
    fn rejects_self_cycle() {
        assert_eq!(
            validate(
                &[c(10)],
                &[Dependency {
                    driver_hash: 10,
                    required_hash: 10
                }]
            ),
            Err(DependencyError::Cycle)
        );
    }
}

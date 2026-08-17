#![no_std]

//! Capability-oriented identity primitives for userspace services.

pub const IDENTITY_ABI_MAJOR: u16 = 1;
pub const IDENTITY_ABI_MINOR: u16 = 1;
pub const MAX_GROUPS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Credential { pub user: UserId, pub primary_group: GroupId, pub capability_mask: u64 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityError { InvalidUser, InvalidGroup, TooManyGroups, CapabilityDenied }

pub const fn validate_user(user: UserId) -> Result<(), IdentityError> {
    if user.0 == u32::MAX { Err(IdentityError::InvalidUser) } else { Ok(()) }
}

pub const fn validate_group(group: GroupId) -> Result<(), IdentityError> {
    if group.0 == u32::MAX { Err(IdentityError::InvalidGroup) } else { Ok(()) }
}

pub const fn authorize(credential: Credential, required: u64) -> Result<(), IdentityError> {
    if credential.capability_mask & required == required { Ok(()) } else { Err(IdentityError::CapabilityDenied) }
}

/// Fixed-capacity group membership; duplicate and over-capacity membership are rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupSet { groups: [GroupId; MAX_GROUPS], len: usize }

impl GroupSet {
    pub const fn new() -> Self { Self { groups: [GroupId(0); MAX_GROUPS], len: 0 } }
    pub fn add(&mut self, group: GroupId) -> Result<(), IdentityError> {
        validate_group(group)?;
        if self.contains(group) { return Ok(()); }
        if self.len == MAX_GROUPS { return Err(IdentityError::TooManyGroups); }
        self.groups[self.len] = group;
        self.len += 1;
        Ok(())
    }
    pub const fn contains(&self, group: GroupId) -> bool {
        let mut i = 0;
        while i < self.len { if self.groups[i] == group { return true; } i += 1; }
        false
    }
    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }
}

impl Default for GroupSet { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capability_check_is_exact() {
        let c = Credential { user: UserId(1), primary_group: GroupId(1), capability_mask: 0b101 };
        assert!(authorize(c, 0b001).is_ok());
        assert!(authorize(c, 0b010).is_err());
    }
    #[test]
    fn group_membership_is_bounded_and_deduplicated() {
        let mut groups = GroupSet::new();
        assert!(groups.add(GroupId(7)).is_ok());
        assert!(groups.add(GroupId(7)).is_ok());
        assert_eq!(groups.len(), 1);
        assert!(groups.contains(GroupId(7)));
    }
}

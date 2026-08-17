#![no_std]

//! Capability-oriented identity primitives for userspace services.

pub const IDENTITY_ABI_MAJOR: u16 = 1;
pub const IDENTITY_ABI_MINOR: u16 = 0;
pub const MAX_GROUPS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UserId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Credential {
    pub user: UserId,
    pub primary_group: GroupId,
    pub capability_mask: u64,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capability_check_is_exact() {
        let c = Credential { user: UserId(1), primary_group: GroupId(1), capability_mask: 0b101 };
        assert!(authorize(c, 0b001).is_ok());
        assert!(authorize(c, 0b010).is_err());
    }
}

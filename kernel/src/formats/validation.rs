//! Shared validation primitives for native AWE binary containers.
//! Dependency-free and `no_std` friendly.

#![allow(dead_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Empty,
    Misaligned,
    TooLarge,
    NonCanonical,
}

pub const MAX_CONTAINER: usize = 16 * 1024 * 1024;

pub fn checked_range(total: usize, start: usize, len: usize) -> Result<(usize, usize), Error> {
    if total == 0 {
        return Err(Error::Empty);
    }
    let end = start.checked_add(len).ok_or(Error::TooLarge)?;
    if end > total {
        return Err(Error::TooLarge);
    }
    Ok((start, end))
}

pub fn require_aligned(value: usize, alignment: usize) -> Result<(), Error> {
    if alignment == 0 || !alignment.is_power_of_two() || value & (alignment - 1) != 0 {
        return Err(Error::Misaligned);
    }
    Ok(())
}

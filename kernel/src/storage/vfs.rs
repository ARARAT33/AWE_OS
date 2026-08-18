//! Bounded VFS contracts for the storage service.
//! The kernel exposes only metadata/handle primitives; concrete filesystem
//! persistence remains in a userspace storage service.

use super::{BLOCK_SIZE, BlockDevice, StorageError};

pub const MAX_NAME: usize = 63;
pub const MAX_INODES: usize = 128;
pub const MAX_JOURNAL: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsError {
    InvalidName,
    NotFound,
    Exists,
    NotDirectory,
    IsDirectory,
    NoSpace,
    InvalidHandle,
    Corrupt,
    ReadOnly,
    Storage(StorageError),
}

impl From<StorageError> for FsError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileName {
    bytes: [u8; MAX_NAME],
    len: u8,
}

impl FileName {
    pub fn new(input: &[u8]) -> Result<Self, FsError> {
        if input.is_empty() || input.len() > MAX_NAME || input.iter().any(|b| *b == b'/' || *b == 0)
        {
            return Err(FsError::InvalidName);
        }
        let mut bytes = [0u8; MAX_NAME];
        bytes[..input.len()].copy_from_slice(input);
        Ok(Self {
            bytes,
            len: input.len() as u8,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeKind {
    File,
    Directory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Inode {
    pub id: u32,
    pub parent: u32,
    pub kind: NodeKind,
    pub size: u64,
    pub generation: u64,
    pub allocated_blocks: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JournalRecord {
    pub sequence: u64,
    pub inode: u32,
    pub block: u64,
    pub old_crc: u32,
    pub new_crc: u32,
    pub committed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    Clean,
    Replay,
    Rollback,
}

pub struct Vfs<const N: usize = MAX_INODES, const J: usize = MAX_JOURNAL> {
    inodes: [Option<Inode>; N],
    names: [Option<(u32, FileName)>; N],
    journal: [Option<JournalRecord>; J],
    next_inode: u32,
    sequence: u64,
}

impl<const N: usize, const J: usize> Vfs<N, J> {
    pub const fn new() -> Self {
        Self {
            inodes: [None; N],
            names: [None; N],
            journal: [None; J],
            next_inode: 1,
            sequence: 1,
        }
    }

    pub fn format(&mut self) -> Result<(), FsError> {
        if N == 0 || J == 0 {
            return Err(FsError::NoSpace);
        }
        self.inodes = [None; N];
        self.names = [None; N];
        self.journal = [None; J];
        self.next_inode = 2;
        self.sequence = 1;
        self.inodes[0] = Some(Inode {
            id: 1,
            parent: 1,
            kind: NodeKind::Directory,
            size: 0,
            generation: 1,
            allocated_blocks: 0,
        });
        Ok(())
    }

    pub fn root(&self) -> Result<Inode, FsError> {
        self.inodes[0].ok_or(FsError::Corrupt)
    }

    pub fn create(&mut self, parent: u32, name: &[u8], kind: NodeKind) -> Result<Inode, FsError> {
        let name = FileName::new(name)?;
        let p = self.find_inode(parent).ok_or(FsError::NotFound)?;
        if p.kind != NodeKind::Directory {
            return Err(FsError::NotDirectory);
        }
        if self
            .names
            .iter()
            .flatten()
            .any(|(pid, n)| *pid == parent && n.as_bytes() == name.as_bytes())
        {
            return Err(FsError::Exists);
        }
        let slot = self
            .inodes
            .iter()
            .position(Option::is_none)
            .ok_or(FsError::NoSpace)?;
        let id = self.next_inode;
        self.next_inode = self.next_inode.checked_add(1).ok_or(FsError::NoSpace)?;
        let inode = Inode {
            id,
            parent,
            kind,
            size: 0,
            generation: 1,
            allocated_blocks: 0,
        };
        self.inodes[slot] = Some(inode);
        self.names[slot] = Some((parent, name));
        Ok(inode)
    }

    pub fn lookup(&self, parent: u32, name: &[u8]) -> Result<Inode, FsError> {
        let name = FileName::new(name)?;
        for i in 0..N {
            if let (Some(inode), Some((pid, stored))) = (self.inodes[i], self.names[i])
                && pid == parent
                && stored.as_bytes() == name.as_bytes()
            {
                return Ok(inode);
            }
        }
        Err(FsError::NotFound)
    }

    pub fn begin_write(
        &mut self,
        inode: u32,
        block: u64,
        old_crc: u32,
        new_crc: u32,
    ) -> Result<u64, FsError> {
        if self.find_inode(inode).is_none() {
            return Err(FsError::NotFound);
        }
        let slot = self
            .journal
            .iter()
            .position(Option::is_none)
            .ok_or(FsError::NoSpace)?;
        let seq = self.sequence;
        self.sequence = self.sequence.checked_add(1).ok_or(FsError::NoSpace)?;
        self.journal[slot] = Some(JournalRecord {
            sequence: seq,
            inode,
            block,
            old_crc,
            new_crc,
            committed: false,
        });
        Ok(seq)
    }

    pub fn commit(&mut self, sequence: u64) -> Result<(), FsError> {
        for record in self.journal.iter_mut().flatten() {
            if record.sequence == sequence {
                record.committed = true;
                return Ok(());
            }
        }
        Err(FsError::NotFound)
    }

    pub fn recovery_action(&self) -> RecoveryAction {
        if self.journal.iter().all(Option::is_none) {
            return RecoveryAction::Clean;
        }
        if self.journal.iter().flatten().any(|r| !r.committed) {
            RecoveryAction::Rollback
        } else {
            RecoveryAction::Replay
        }
    }

    pub fn fsck(&self) -> Result<(), FsError> {
        let root = self.root()?;
        if root.kind != NodeKind::Directory || root.id != root.parent {
            return Err(FsError::Corrupt);
        }
        for inode in self.inodes.iter().flatten() {
            if inode.id != 1 && self.find_inode(inode.parent).is_none() {
                return Err(FsError::Corrupt);
            }
        }
        Ok(())
    }

    fn find_inode(&self, id: u32) -> Option<Inode> {
        self.inodes.iter().flatten().find(|i| i.id == id).copied()
    }
}

impl<const N: usize, const J: usize> Default for Vfs<N, J> {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a bounded filesystem block transfer without allocating.
pub fn validate_io<D: BlockDevice>(
    device: &D,
    block: u64,
    offset: usize,
    len: usize,
) -> Result<(), FsError> {
    if offset > BLOCK_SIZE || len > BLOCK_SIZE.saturating_sub(offset) {
        return Err(FsError::Storage(StorageError::BufferTooSmall));
    }
    if block >= device.block_count() {
        return Err(FsError::Storage(StorageError::InvalidBlock));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::RamBlockDevice;

    #[test]
    fn vfs_is_bounded_and_recoverable() {
        let mut fs = Vfs::<8, 4>::new();
        fs.format().unwrap();
        let file = fs.create(1, b"hello", NodeKind::File).unwrap();
        assert_eq!(fs.lookup(1, b"hello").unwrap(), file);
        let seq = fs.begin_write(file.id, 3, 1, 2).unwrap();
        assert_eq!(fs.recovery_action(), RecoveryAction::Rollback);
        fs.commit(seq).unwrap();
        assert_eq!(fs.recovery_action(), RecoveryAction::Replay);
        assert!(fs.fsck().is_ok());
    }

    #[test]
    fn io_bounds_fail_closed() {
        let disk = RamBlockDevice::default();
        assert!(validate_io(&disk, 0, BLOCK_SIZE - 8, 8).is_ok());
        assert!(validate_io(&disk, 0, BLOCK_SIZE - 7, 8).is_err());
        assert!(validate_io(&disk, disk.block_count(), 0, 1).is_err());
    }
}

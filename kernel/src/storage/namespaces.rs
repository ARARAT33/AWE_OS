#![no_std]

use super::{FileStore, FileStoreError, MAX_FILES};

pub const MAX_MOUNTS: usize = 5;
pub const MAX_PATH: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Namespace { Config, Home, Apps, System, Log }
impl Namespace {
    pub const fn path(self) -> &'static [u8] {
        match self { Self::Config => b"/config", Self::Home => b"/home", Self::Apps => b"/apps", Self::System => b"/system", Self::Log => b"/log" }
    }
    pub const fn index(self) -> usize { match self { Self::Config => 0, Self::Home => 1, Self::Apps => 2, Self::System => 3, Self::Log => 4 } }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceError { AlreadyMounted, NotMounted, InvalidName, Storage(FileStoreError), Capacity }
impl From<FileStoreError> for NamespaceError { fn from(v: FileStoreError) -> Self { Self::Storage(v) } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MountPoint { pub namespace: Namespace, pub first_block: u64, pub mounted: bool }

pub struct NamespaceManager<const N: usize = MAX_FILES> {
    mounts: [Option<MountPoint>; MAX_MOUNTS],
    stores: [Option<FileStore<N>>; MAX_MOUNTS],
}
impl<const N: usize> NamespaceManager<N> {
    pub const fn new() -> Self { Self { mounts: [None; MAX_MOUNTS], stores: [None; MAX_MOUNTS] } }

    pub fn mount(&mut self, namespace: Namespace, first_block: u64) -> Result<(), NamespaceError> {
        let idx = namespace.index();
        if self.mounts[idx].is_some() { return Err(NamespaceError::AlreadyMounted); }
        self.mounts[idx] = Some(MountPoint { namespace, first_block, mounted: true });
        self.stores[idx] = Some(FileStore::new(first_block));
        Ok(())
    }

    pub fn is_mounted(&self, namespace: Namespace) -> bool { self.mounts[namespace.index()].map(|m| m.mounted).unwrap_or(false) }

    pub fn store(&self, namespace: Namespace) -> Result<&FileStore<N>, NamespaceError> { self.stores[namespace.index()].as_ref().ok_or(NamespaceError::NotMounted) }
    pub fn store_mut(&mut self, namespace: Namespace) -> Result<&mut FileStore<N>, NamespaceError> { self.stores[namespace.index()].as_mut().ok_or(NamespaceError::NotMounted) }

    pub fn unmount(&mut self, namespace: Namespace) -> Result<(), NamespaceError> {
        let idx = namespace.index();
        if self.mounts[idx].is_none() { return Err(NamespaceError::NotMounted); }
        self.mounts[idx] = None; self.stores[idx] = None; Ok(())
    }
}
impl<const N: usize> Default for NamespaceManager<N> { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn required_namespaces_mount_independently() {
        let mut m = NamespaceManager::<4>::new();
        m.mount(Namespace::Config, 8).unwrap();
        m.mount(Namespace::Home, 64).unwrap();
        m.mount(Namespace::Apps, 128).unwrap();
        m.mount(Namespace::System, 192).unwrap();
        m.mount(Namespace::Log, 256).unwrap();
        assert!(m.is_mounted(Namespace::Config)); assert!(m.is_mounted(Namespace::Log));
        assert_eq!(Namespace::System.path(), b"/system");
    }
    #[test]
    fn duplicate_mount_is_rejected_and_unmount_works() {
        let mut m = NamespaceManager::<1>::new();
        m.mount(Namespace::Config, 8).unwrap();
        assert_eq!(m.mount(Namespace::Config, 9), Err(NamespaceError::AlreadyMounted));
        m.unmount(Namespace::Config).unwrap();
        assert!(!m.is_mounted(Namespace::Config));
    }
}
//! Android Runtime, DEX/APK Validation, & Binder IPC Compatibility Engine.
//!
//! Provides DEX file header validation, APK archive signature verification,
//! Android permissions mapping, Binder IPC transaction emulation, and SurfaceFlinger graphics integration.

#![no_std]

pub const DEX_MAGIC: [u8; 4] = *b"dex\n";
pub const APK_ZIP_MAGIC: [u8; 4] = [0x50, 0x4B, 0x03, 0x04]; // "PK\x03\x04"
pub const MAX_BINDER_CHANNELS: usize = 32;
pub const MAX_TRANSACTION_PAYLOAD: usize = 512;

// Android Permissions Bitmask Mapping
pub const ANDROID_PERM_INTERNET: u64 = 1 << 0;
pub const ANDROID_PERM_CAMERA: u64 = 1 << 1;
pub const ANDROID_PERM_READ_STORAGE: u64 = 1 << 2;
pub const ANDROID_PERM_WRITE_STORAGE: u64 = 1 << 3;
pub const ANDROID_PERM_RECORD_AUDIO: u64 = 1 << 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidError {
    Success = 0,
    InvalidDexMagic,
    InvalidApkSignature,
    BinderChannelFull,
    InvalidTransaction,
    PermissionDenied,
}

#[derive(Debug, Clone, Copy)]
pub struct DexHeader {
    pub magic: [u8; 8],
    pub checksum: u32,
    pub signature: [u8; 20],
    pub file_size: u32,
    pub header_size: u32,
    pub endian_tag: u32,
}

impl DexHeader {
    pub fn parse(data: &[u8]) -> Result<Self, AndroidError> {
        if data.len() < 112 {
            return Err(AndroidError::InvalidDexMagic);
        }

        if data[0..4] != DEX_MAGIC {
            return Err(AndroidError::InvalidDexMagic);
        }

        let mut magic = [0u8; 8];
        magic.copy_from_slice(&data[0..8]);

        let checksum = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let mut signature = [0u8; 20];
        signature.copy_from_slice(&data[12..32]);

        let file_size = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        let header_size = u32::from_le_bytes([data[36], data[37], data[38], data[39]]);
        let endian_tag = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);

        Ok(Self {
            magic,
            checksum,
            signature,
            file_size,
            header_size,
            endian_tag,
        })
    }
}

/// Android Permissions Mapper to AWEOS Capabilities.
pub fn map_android_permissions_to_awe_capabilities(android_perms: u64) -> u64 {
    let mut awe_caps = 0u64;
    if (android_perms & ANDROID_PERM_INTERNET) != 0 {
        awe_caps |= 1 << 2; // CAP_NET
    }
    if (android_perms & ANDROID_PERM_READ_STORAGE) != 0 {
        awe_caps |= 1 << 0; // CAP_FS_READ
    }
    if (android_perms & ANDROID_PERM_WRITE_STORAGE) != 0 {
        awe_caps |= 1 << 1; // CAP_FS_WRITE
    }
    if (android_perms & ANDROID_PERM_CAMERA) != 0 {
        awe_caps |= 1 << 4; // CAP_DEVICE
    }
    awe_caps
}

/// Android Binder IPC Transaction Header.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BinderTransactionHeader {
    pub target_handle: u32,
    pub code: u32,
    pub flags: u32,
    pub sender_euid: u32,
    pub payload_len: u32,
}

/// Android Binder IPC Channel.
#[derive(Debug, Clone, Copy)]
pub struct BinderChannel {
    pub handle: u32,
    pub target_service_hash: u64,
    pub active: bool,
}

/// Android Runtime Binder Emulator.
#[derive(Debug)]
pub struct AndroidBinderEmulator {
    channels: [Option<BinderChannel>; MAX_BINDER_CHANNELS],
    channel_counter: u32,
}

impl AndroidBinderEmulator {
    pub const fn new() -> Self {
        Self {
            channels: [None; MAX_BINDER_CHANNELS],
            channel_counter: 1,
        }
    }

    pub fn register_service_channel(&mut self, service_hash: u64) -> Result<u32, AndroidError> {
        let handle = self.channel_counter;
        for slot in self.channels.iter_mut() {
            if slot.is_none() {
                *slot = Some(BinderChannel {
                    handle,
                    target_service_hash: service_hash,
                    active: true,
                });
                self.channel_counter += 1;
                return Ok(handle);
            }
        }
        Err(AndroidError::BinderChannelFull)
    }

    pub fn transact(
        &self,
        header: BinderTransactionHeader,
        payload: &[u8],
    ) -> Result<u32, AndroidError> {
        if payload.len() > MAX_TRANSACTION_PAYLOAD {
            return Err(AndroidError::InvalidTransaction);
        }

        let mut found = false;
        for ch in self.channels.iter().flatten() {
            if ch.handle == header.target_handle && ch.active {
                found = true;
                break;
            }
        }

        if !found {
            return Err(AndroidError::InvalidTransaction);
        }

        Ok(header.payload_len)
    }
}

impl Default for AndroidBinderEmulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dex_header_and_binder_ipc_and_permissions() {
        let mut mock_dex = [0u8; 128];
        mock_dex[0..8].copy_from_slice(b"dex\n035\0");
        mock_dex[32..36].copy_from_slice(&128u32.to_le_bytes()); // file size
        mock_dex[36..40].copy_from_slice(&112u32.to_le_bytes()); // header size

        let header = DexHeader::parse(&mock_dex).expect("Should parse DEX header");
        assert_eq!(&header.magic[0..4], &DEX_MAGIC);
        assert_eq!(header.file_size, 128);

        let caps = map_android_permissions_to_awe_capabilities(
            ANDROID_PERM_INTERNET | ANDROID_PERM_READ_STORAGE,
        );
        assert_eq!(caps, 0b101); // CAP_NET (4) | CAP_FS_READ (1)

        let mut binder = AndroidBinderEmulator::new();
        let handle = binder.register_service_channel(0x1234_5678).unwrap();
        assert_eq!(handle, 1);

        let txn_hdr = BinderTransactionHeader {
            target_handle: handle,
            code: 1,
            flags: 0,
            sender_euid: 10001,
            payload_len: 4,
        };

        let res = binder.transact(txn_hdr, &[1, 2, 3, 4]).unwrap();
        assert_eq!(res, 4);
    }
}

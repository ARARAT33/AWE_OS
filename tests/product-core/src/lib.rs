#[cfg(test)]
mod product_core {
    use awe_appd::{AWOS_HEADER_LEN, AWOS_MAGIC, AWOS_VERSION, AppPackageState, validate_awos};
    use awe_driverd::{
        ASD_HEADER_LEN, ASD_MAGIC, ASD_VERSION, AsdError, PackageState, validate_asd,
    };
    use awe_update::{Slot, SlotState, UpdateError, UpdateManager, UpdateManifest, Version};
    use aweos_kernel::net::{Endpoint, Ipv4Address, SocketTable, Transport};
    use aweos_kernel::storage::{
        BLOCK_SIZE, BlockDevice, NodeKind, RamBlockDevice, RecoveryAction, Vfs,
    };

    fn asd_package(manifest: usize, payload: usize, signature: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(ASD_HEADER_LEN + manifest + payload + signature);
        bytes.extend_from_slice(&ASD_MAGIC);
        bytes.extend_from_slice(&ASD_VERSION.to_le_bytes());
        bytes.extend_from_slice(&0x8664u16.to_le_bytes());
        bytes.extend_from_slice(&(manifest as u32).to_le_bytes());
        bytes.extend_from_slice(&(payload as u32).to_le_bytes());
        bytes.extend_from_slice(&(signature as u16).to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.extend_from_slice(&0u64.to_le_bytes());
        bytes.resize(ASD_HEADER_LEN + manifest + payload + signature, 0);
        bytes
    }

    fn awos_package(manifest: usize, code: usize, data: usize, signature: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(AWOS_HEADER_LEN + manifest + code + data + signature);
        bytes.extend_from_slice(&AWOS_MAGIC);
        bytes.extend_from_slice(&AWOS_VERSION.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&(manifest as u32).to_le_bytes());
        bytes.extend_from_slice(&(code as u32).to_le_bytes());
        bytes.extend_from_slice(&(data as u32).to_le_bytes());
        bytes.extend_from_slice(&(signature as u16).to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.resize(AWOS_HEADER_LEN + manifest + code + data + signature, 0);
        bytes
    }

    fn manifest(generation: u64) -> UpdateManifest {
        UpdateManifest {
            version: Version::new(b"1.0.0").expect("valid version"),
            generation,
            payload_len: 4096,
            payload_digest: [0xA5; 32],
            min_generation: generation,
        }
    }

    #[test]
    fn asd_admission_is_exercised_end_to_end() {
        let package = asd_package(16, 128, 64);
        assert!(validate_asd(&package).is_ok());

        let mut malformed = package.clone();
        malformed[16..18].copy_from_slice(&63u16.to_le_bytes());
        assert_eq!(validate_asd(&malformed), Err(AsdError::MissingSignature));

        assert!(awe_driverd::package_transition(
            PackageState::Installed,
            PackageState::Active
        ));
        assert!(awe_driverd::package_transition(
            PackageState::Active,
            PackageState::Staged
        ));
        assert!(awe_driverd::package_transition(
            PackageState::Staged,
            PackageState::Failed
        ));
        assert!(!awe_driverd::package_transition(
            PackageState::Installed,
            PackageState::Failed
        ));
    }

    #[test]
    fn awos_admission_and_lifecycle_are_exercised_end_to_end() {
        let package = awos_package(16, 256, 32, 64);
        assert!(validate_awos(&package).is_ok());

        let mut malformed = package.clone();
        malformed[24..28].copy_from_slice(&256u32.to_le_bytes());
        assert!(validate_awos(&malformed).is_err());

        assert!(awe_appd::package_transition(
            AppPackageState::Installed,
            AppPackageState::Running
        ));
        assert!(awe_appd::package_transition(
            AppPackageState::Running,
            AppPackageState::Failed
        ));
        assert!(awe_appd::package_transition(
            AppPackageState::Failed,
            AppPackageState::Quarantined
        ));
        assert!(awe_appd::package_transition(
            AppPackageState::Installed,
            AppPackageState::Removed
        ));
    }

    #[test]
    fn ab_update_boot_health_and_rollback_are_exercised_end_to_end() {
        let mut manager = UpdateManager::new(10);
        manager.stage(Slot::B, manifest(11)).expect("stage update");
        assert_eq!(manager.boot_pending().expect("boot pending"), Slot::B);
        assert_eq!(manager.state(Slot::B), SlotState::Booting);
        manager.mark_failed(Slot::B).expect("mark failed");
        manager.rollback(Slot::B).expect("rollback");
        assert_eq!(manager.active(), Slot::A);
        assert_eq!(manager.generation(), 10);

        manager
            .stage(Slot::B, manifest(11))
            .expect("restage update");
        manager.boot_pending().expect("boot pending");
        manager.mark_healthy(Slot::B).expect("healthy boot");
        assert_eq!(manager.active(), Slot::B);
        assert_eq!(manager.generation(), 11);
    }

    #[test]
    fn downgrade_is_rejected_by_the_runtime_update_path() {
        let mut manager = UpdateManager::new(20);
        assert_eq!(
            manager.stage(Slot::B, manifest(19)),
            Err(UpdateError::Downgrade)
        );
    }

    #[test]
    fn persistent_storage_runtime_round_trip_and_recovery() {
        let mut disk = RamBlockDevice::default();
        let mut block = [0u8; BLOCK_SIZE];
        block[0] = 0xA5;
        block[BLOCK_SIZE - 1] = 0x5A;
        disk.write_block(7, &block).expect("persistent write");
        assert!(disk.is_dirty());

        let mut readback = [0u8; BLOCK_SIZE];
        disk.read_block(7, &mut readback).expect("persistent read");
        assert_eq!(readback, block);
        disk.flush().expect("flush");
        assert!(!disk.is_dirty());

        let mut vfs = Vfs::<16, 8>::new();
        vfs.format().expect("format");
        let file = vfs
            .create(1, b"runtime.log", NodeKind::File)
            .expect("create");
        let sequence = vfs
            .begin_write(file.id, 7, 0x1111, 0x2222)
            .expect("journal begin");
        assert_eq!(vfs.recovery_action(), RecoveryAction::Rollback);
        vfs.commit(sequence).expect("journal commit");
        assert_eq!(vfs.recovery_action(), RecoveryAction::Replay);
        vfs.fsck().expect("fsck");
    }

    #[test]
    fn network_runtime_socket_and_packet_validation() {
        let mut sockets = SocketTable::<4>::new();
        let local = Endpoint::new(Ipv4Address::LOOPBACK, 8080);
        let remote = Endpoint::new(Ipv4Address::LOOPBACK, 443);
        let slot = sockets.bind(local, Transport::Tcp).expect("bind TCP");
        sockets.connect(slot, remote).expect("connect TCP");
        assert!(sockets.get(slot).expect("socket").connected);
        assert!(sockets.bind(local, Transport::Tcp).is_err());

        let udp = [0x1F, 0x90, 0x01, 0xBB, 0x00, 0x0C, 0, 0, 1, 2, 3, 4];
        let (src, dst, payload) =
            aweos_kernel::net::transport::udp_payload(&udp).expect("UDP");
        assert_eq!(src.port, 8080);
        assert_eq!(dst.port, 443);
        assert_eq!(payload, &[1, 2, 3, 4]);

        let mut tcp = [0u8; 20];
        tcp[0..2].copy_from_slice(&8080u16.to_be_bytes());
        tcp[2..4].copy_from_slice(&443u16.to_be_bytes());
        tcp[12] = 5 << 4;
        assert_eq!(
            aweos_kernel::net::transport::tcp_header_valid(&tcp).expect("TCP"),
            20
        );
        assert!(aweos_kernel::net::transport::udp_payload(&[0; 7]).is_err());
    }
}

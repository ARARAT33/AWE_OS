#[cfg(test)]
mod product_core {
    use awe_appd::{AWOS_HEADER_LEN, AWOS_MAGIC, AWOS_VERSION, AppPackageState, validate_awos};
    use awe_driverd::{
        ASD_HEADER_LEN, ASD_MAGIC, ASD_VERSION, AsdError, PackageState, validate_asd,
    };
    use awe_update::{Slot, SlotState, UpdateManager, UpdateManifest, Version};
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
    fn full_awe_nexus_services_and_compatibility_integration() {
        use awe_netd::{NetworkDaemon, SocketProtocol};
        use awe_nexus::{NexusHeader, NexusMessage, NexusRouter, ServiceEndpoint};
        use awe_securityd::SecurityDaemon;
        use awe_storaged::{StorageDaemon, VolumeType};

        // 1. AWE-Nexus Router
        let mut router = NexusRouter::new();
        router
            .register_service(ServiceEndpoint::new(1, 0b111))
            .unwrap();
        router
            .register_service(ServiceEndpoint::new(2, 0b111))
            .unwrap();

        let hdr = NexusHeader {
            sender_id: 1,
            receiver_id: 2,
            opcode: 0x10,
            payload_len: 4,
            sequence_num: 1,
            required_capability: 0b001,
        };
        let msg = NexusMessage::new(hdr, &[1, 2, 3, 4]).unwrap();
        assert!(router.send_message(1, msg).is_ok());
        assert!(router.receive_message(2).is_some());

        // 2. Netd
        let mut netd = NetworkDaemon::new();
        let sock = netd.create_socket(SocketProtocol::Tcp, 100).unwrap();
        assert_eq!(sock, 1);

        // 3. Storaged
        let mut storaged = StorageDaemon::new();
        let vol = storaged
            .register_volume(VolumeType::Ramdisk, 1024, 0, false)
            .unwrap();
        assert_eq!(vol, 1);

        // 4. Securityd
        let mut sec = SecurityDaemon::new([0x33; 16]);
        let tok = sec.issue_token(100, 0b101, 500, 200).unwrap();
        assert!(sec.validate_token(tok.token_id, 100, 0b001, 600));
    }

    #[test]
    fn kernel_compatibility_runtimes_end_to_end() {
        use aweos_kernel::compat::android::AndroidBinderEmulator;
        use aweos_kernel::compat::linux::LinuxSyscallDispatcher;
        use aweos_kernel::compat::windows::{NtStatus, Win32SyscallDispatcher};
        use aweos_kernel::compat::wlin::WlinBridge;

        // Windows
        let mut win = Win32SyscallDispatcher::new();
        assert_eq!(win.dispatch(0x0055, 0x9999, 0x01, 0), NtStatus::Success);

        // Linux
        let mut lin = LinuxSyscallDispatcher::new();
        assert_eq!(lin.dispatch(1, 1, 0, 0).unwrap(), 0);

        // Android Binder
        let mut binder = AndroidBinderEmulator::new();
        let ch = binder.register_service_channel(0x1122).unwrap();
        assert_eq!(ch, 1);

        // WLIN Bridge
        let mut wlin = WlinBridge::new();
        let rid = wlin.map_cross_runtime_resource(4, 3, 1024).unwrap();
        assert_eq!(rid, 1);
    }
}

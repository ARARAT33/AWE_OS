#[cfg(test)]
mod product_core {
    use awe_appd::{AWOS_HEADER_LEN, AWOS_MAGIC, AWOS_VERSION, AppPackageState, validate_awos};
    use awe_awosa::{
        AbiVersion, CAP_FS_READ, CAP_IPC, IoKind, RuntimeError, negotiate, validate_io,
    };
    use awe_driverd::{
        ASD_HEADER_LEN, ASD_MAGIC, ASD_VERSION, AsdError, PackageState, validate_asd,
    };
    use awe_identityd::{Credential, GroupId, GroupSet, UserId, authorize};
    use awe_initd::{RestartPolicy, ServiceId, ServiceSpec, ServiceState, ServiceTable};
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
        let (src, dst, payload) = aweos_kernel::net::transport::udp_payload(&udp).expect("UDP");
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

    #[test]
    fn stages_a_to_j_cross_service_contracts_are_bounded_and_fail_closed() {
        assert!(negotiate(AbiVersion { major: 2, minor: 0 }).is_err());
        assert!(negotiate(AbiVersion { major: 1, minor: 2 }).is_ok());

        assert!(validate_io(IoKind::Read, 4096, CAP_FS_READ).is_ok());
        assert_eq!(
            validate_io(IoKind::Write, 128, CAP_FS_READ),
            Err(RuntimeError::CapabilityDenied)
        );

        let credential = Credential {
            user: UserId(1000),
            primary_group: GroupId(1000),
            capability_mask: CAP_FS_READ | CAP_IPC,
        };
        assert!(authorize(credential, CAP_FS_READ).is_ok());
        assert!(authorize(credential, 1 << 20).is_err());
        let mut groups = GroupSet::new();
        groups.add(GroupId(1000)).expect("group");
        groups.add(GroupId(1000)).expect("deduplicated group");
        assert_eq!(groups.len(), 1);

        let mut disk = RamBlockDevice::default();
        let block = [0x5Au8; BLOCK_SIZE];
        disk.write_block(3, &block).expect("block write");
        let mut readback = [0u8; BLOCK_SIZE];
        disk.read_block(3, &mut readback).expect("block read");
        assert_eq!(readback, block);

        let mut sockets = SocketTable::<2>::new();
        let slot = sockets
            .bind(Endpoint::new(Ipv4Address::LOOPBACK, 5353), Transport::Udp)
            .expect("UDP bind");
        assert!(sockets.get(slot).is_some());

        let spec = ServiceSpec {
            id: ServiceId(2),
            restart: RestartPolicy::OnFailure,
            capability_mask: CAP_FS_READ,
            memory_limit_pages: 8,
            cpu_budget_ticks: 100,
        };
        let mut services = ServiceTable::new();
        services.register(spec).expect("register service");
        services
            .set_state(ServiceId(2), ServiceState::Starting)
            .expect("start");
        services
            .set_state(ServiceId(2), ServiceState::Running)
            .expect("run");
        services
            .set_state(ServiceId(2), ServiceState::Failed)
            .expect("fail");
        services.restart(ServiceId(2)).expect("restart");
        assert_eq!(services.state(ServiceId(2)), Some(ServiceState::Starting));

        assert!(validate_io(IoKind::Message, 128, CAP_IPC).is_ok());
        assert_eq!(
            validate_io(IoKind::Message, 128, 0),
            Err(RuntimeError::CapabilityDenied)
        );

        let app = awos_package(16, 64, 16, 64);
        assert!(validate_awos(&app).is_ok());
        assert_eq!(awe_awosa::required_capability(IoKind::Read), CAP_FS_READ);
    }

    #[test]
    fn app_ui_and_kernel_boundaries_are_exercised_together() {
        use awe_appd::{AppId, AppManifest, validate_declarations, validate_manifest};
        use awe_ayui::{Compositor, Rect};
        use aweos_kernel::process::ProcessState;
        use aweos_kernel::time::MonotonicClock;

        let manifest = AppManifest {
            id: AppId(1),
            abi_major: awe_appd::AWE_APP_ABI_MAJOR,
            abi_minor: awe_appd::AWE_APP_ABI_MINOR,
            memory_limit_pages: 16,
            capability_mask: 0,
            dependency_count: 2,
            resource_count: 2,
        };
        assert!(validate_manifest(manifest).is_ok());
        assert!(validate_declarations(2, 2).is_ok());

        let mut compositor = Compositor::new();
        let window = compositor
            .create_window(Rect {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            })
            .expect("valid window");
        compositor.focus(window).expect("focus");
        assert_eq!(compositor.window_count(), 1);

        assert!(ProcessState::Created.can_transition(ProcessState::Runnable));
        assert!(!ProcessState::Created.can_transition(ProcessState::Running));
        let mut clock = MonotonicClock::new();
        clock.advance(10);
        assert_eq!(clock.now().0, 10);
    }

    #[test]
    fn app_ui_budget_overflow_fails_closed() {
        use awe_appd::{AppError, MAX_DEPS, MAX_RESOURCES, validate_declarations};
        use awe_ayui::{Compositor, MAX_HEIGHT, MAX_WIDTH, Rect, UiError};

        assert_eq!(
            validate_declarations(MAX_DEPS + 1, 0),
            Err(AppError::TooManyDependencies)
        );
        assert_eq!(
            validate_declarations(0, MAX_RESOURCES + 1),
            Err(AppError::TooManyResources)
        );

        let mut compositor = Compositor::new();
        assert_eq!(
            compositor.create_window(Rect {
                x: 0,
                y: 0,
                width: MAX_WIDTH + 1,
                height: MAX_HEIGHT
            }),
            Err(UiError::InvalidRect)
        );
    }
}

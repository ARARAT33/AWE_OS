#[cfg(test)]
mod product_core {
    use awe_appd::{validate_awos, AppPackageState, AWOS_HEADER_LEN, AWOS_MAGIC, AWOS_VERSION};
    use awe_driverd::{validate_asd, AsdError, PackageState, ASD_HEADER_LEN, ASD_MAGIC, ASD_VERSION};
    use awe_update::{Slot, SlotState, UpdateManager, UpdateManifest, Version};

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

        assert!(awe_driverd::package_transition(PackageState::Installed, PackageState::Active));
        assert!(awe_driverd::package_transition(PackageState::Active, PackageState::Staged));
        assert!(awe_driverd::package_transition(PackageState::Staged, PackageState::Failed));
        assert!(!awe_driverd::package_transition(PackageState::Installed, PackageState::Failed));
    }

    #[test]
    fn awos_admission_and_lifecycle_are_exercised_end_to_end() {
        let package = awos_package(16, 256, 32, 64);
        assert!(validate_awos(&package).is_ok());

        let mut malformed = package.clone();
        malformed[24..28].copy_from_slice(&256u32.to_le_bytes());
        assert!(validate_awos(&malformed).is_err());

        assert!(awe_appd::package_transition(AppPackageState::Installed, AppPackageState::Running));
        assert!(awe_appd::package_transition(AppPackageState::Running, AppPackageState::Failed));
        assert!(awe_appd::package_transition(AppPackageState::Failed, AppPackageState::Quarantined));
        assert!(awe_appd::package_transition(AppPackageState::Installed, AppPackageState::Removed));
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

        manager.stage(Slot::B, manifest(11)).expect("restage update");
        manager.boot_pending().expect("boot pending");
        manager.mark_healthy(Slot::B).expect("healthy boot");
        assert_eq!(manager.active(), Slot::B);
        assert_eq!(manager.generation(), 11);
    }

    #[test]
    fn downgrade_is_rejected_by_the_runtime_update_path() {
        let mut manager = UpdateManager::new(20);
        assert_eq!(manager.stage(Slot::B, manifest(19)), Err(awe_update::UpdateError::Downgrade));
    }
}

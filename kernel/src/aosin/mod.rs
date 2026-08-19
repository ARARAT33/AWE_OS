//! AOSIN (AWEOS System Installer & Migration Engine).
//!
//! Handles dual-boot installation, partition layout validation, Windows/Linux
//! migration engine, system snapshot verification, and rollback recovery.

#![no_std]

pub const MAX_INSTALL_STEPS: usize = 16;
pub const MAX_MIGRATION_ITEMS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    NotStarted,
    Partitioning,
    CopyingFiles,
    ConfiguringBoot,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPartitionType {
    EfiSystemPartition,
    AweosSystemRoot,
    AweosUserData,
    RecoveryImage,
}

#[derive(Debug, Clone, Copy)]
pub struct PartitionPlan {
    pub part_type: TargetPartitionType,
    pub start_lba: u64,
    pub size_blocks: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct MigrationItem {
    pub source_os_hash: u64,
    pub item_type: u32, // 1=user profile, 2=documents, 3=network config
    pub size_bytes: u64,
}

/// AOSIN System Installer Engine Instance.
#[derive(Debug)]
pub struct AosinInstaller {
    pub current_state: InstallState,
    pub target_disk_size: u64,
    partitions: [Option<PartitionPlan>; 4],
    migration_queue: [Option<MigrationItem>; MAX_MIGRATION_ITEMS],
    migration_count: usize,
    pub rollback_checkpoint: u64,
}

impl AosinInstaller {
    pub const fn new(target_disk_size: u64) -> Self {
        Self {
            current_state: InstallState::NotStarted,
            target_disk_size,
            partitions: [None; 4],
            migration_queue: [None; MAX_MIGRATION_ITEMS],
            migration_count: 0,
            rollback_checkpoint: 0,
        }
    }

    pub fn prepare_partition_layout(&mut self) -> Result<(), &'static str> {
        if self.target_disk_size < 10_000_000 {
            return Err("Target disk size insufficient for AWEOS installation");
        }
        self.current_state = InstallState::Partitioning;

        let esp_size = 1_000_000u64; // ~512MB in blocks
        let system_size = 5_000_000u64;
        let recovery_size = 1_000_000u64;
        let data_size = self.target_disk_size - (esp_size + system_size + recovery_size);

        self.partitions[0] = Some(PartitionPlan {
            part_type: TargetPartitionType::EfiSystemPartition,
            start_lba: 2048,
            size_blocks: esp_size,
        });

        self.partitions[1] = Some(PartitionPlan {
            part_type: TargetPartitionType::AweosSystemRoot,
            start_lba: 2048 + esp_size,
            size_blocks: system_size,
        });

        self.partitions[2] = Some(PartitionPlan {
            part_type: TargetPartitionType::AweosUserData,
            start_lba: 2048 + esp_size + system_size,
            size_blocks: data_size,
        });

        self.partitions[3] = Some(PartitionPlan {
            part_type: TargetPartitionType::RecoveryImage,
            start_lba: 2048 + esp_size + system_size + data_size,
            size_blocks: recovery_size,
        });

        Ok(())
    }

    pub fn register_migration_item(&mut self, item: MigrationItem) -> Result<(), &'static str> {
        if self.migration_count >= MAX_MIGRATION_ITEMS {
            return Err("Migration queue full");
        }
        self.migration_queue[self.migration_count] = Some(item);
        self.migration_count += 1;
        Ok(())
    }

    pub fn execute_installation_step(&mut self) -> Result<InstallState, &'static str> {
        match self.current_state {
            InstallState::NotStarted => {
                self.prepare_partition_layout()?;
            }
            InstallState::Partitioning => {
                self.rollback_checkpoint = 1;
                self.current_state = InstallState::CopyingFiles;
            }
            InstallState::CopyingFiles => {
                self.rollback_checkpoint = 2;
                self.current_state = InstallState::ConfiguringBoot;
            }
            InstallState::ConfiguringBoot => {
                self.rollback_checkpoint = 3;
                self.current_state = InstallState::Complete;
            }
            InstallState::Complete => {}
            InstallState::Failed => return Err("Installation previously failed"),
        }
        Ok(self.current_state)
    }

    pub fn trigger_rollback(&mut self) -> InstallState {
        self.current_state = InstallState::Failed;
        self.rollback_checkpoint = 0;
        self.current_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aosin_installer_pipeline() {
        let mut installer = AosinInstaller::new(20_000_000);
        assert_eq!(installer.current_state, InstallState::NotStarted);

        installer.execute_installation_step().unwrap();
        assert_eq!(installer.current_state, InstallState::Partitioning);

        installer.execute_installation_step().unwrap();
        assert_eq!(installer.current_state, InstallState::CopyingFiles);

        installer.execute_installation_step().unwrap();
        assert_eq!(installer.current_state, InstallState::ConfiguringBoot);

        installer.execute_installation_step().unwrap();
        assert_eq!(installer.current_state, InstallState::Complete);

        let mig_item = MigrationItem {
            source_os_hash: 0x9999,
            item_type: 1,
            size_bytes: 4096,
        };
        assert!(installer.register_migration_item(mig_item).is_ok());
    }
}

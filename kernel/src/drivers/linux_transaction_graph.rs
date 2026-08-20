#![no_std]
use super::bus::DeviceId;
use super::contract::HardwareResource;
use super::linux_dependency::{Dependency, DependencyError};
use super::linux_dependency_graph::{GraphError, validate_graph};
use super::linux_package::LinuxPackageHeader;
use super::linux_resolver::LinuxCandidate;
use super::linux_runtime::LinuxRuntime;
use super::linux_transaction::{DriverTransaction, TransactionError};
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GraphTransactionError {
    Dependency(DependencyError),
    Graph(GraphError),
    Transaction(TransactionError),
}
/// Validates the full dependency graph, then prepares the unique top-level
/// candidate. Dependencies are graph metadata and must not make the resolver
/// report an ambiguity between the root driver and its prerequisites.
pub fn prepare_graph_guarded(
    device: DeviceId,
    candidates: &[LinuxCandidate],
    dependencies: &[Dependency],
    runtime: &LinuxRuntime,
    header: LinuxPackageHeader,
    resources: HardwareResource,
) -> Result<DriverTransaction, GraphTransactionError> {
    super::linux_dependency::validate(candidates, dependencies)
        .map_err(GraphTransactionError::Dependency)?;
    let mut nodes = [0u64; 128];
    let mut count = 0usize;
    for candidate in candidates.iter().take(nodes.len()) {
        nodes[count] = candidate.descriptor.module_hash;
        count += 1
    }
    validate_graph(&nodes[..count], dependencies).map_err(GraphTransactionError::Graph)?;
    let mut root = None;
    let mut roots = 0usize;
    for candidate in candidates {
        let hash = candidate.descriptor.module_hash;
        let is_driver = dependencies.iter().any(|d| d.driver_hash == hash);
        let is_required = dependencies.iter().any(|d| d.required_hash == hash);
        if is_driver && !is_required {
            root = Some(*candidate);
            roots += 1
        }
    }
    if roots == 1 {
        let selected = [root.unwrap()];
        DriverTransaction::prepare(device, &selected, runtime, header, resources)
            .map_err(GraphTransactionError::Transaction)
    } else {
        DriverTransaction::prepare(device, candidates, runtime, header, resources)
            .map_err(GraphTransactionError::Transaction)
    }
}
#[cfg(test)]
mod tests {
    use super::super::linux_package::{LDRIVER_MAGIC, LinuxDriverDescriptor};
    use super::*;
    const DEV: DeviceId = DeviceId {
        vendor: 0x8086,
        device: 0x100e,
        class: 0x0200,
        revision: 1,
    };
    const RES: HardwareResource = HardwareResource {
        mmio_base: 0x1000,
        mmio_length: 0x1000,
        dma_mask: u64::MAX,
        irq: 11,
    };
    const HDR: LinuxPackageHeader = LinuxPackageHeader {
        magic: LDRIVER_MAGIC,
        format_version: 1,
        descriptor_size: core::mem::size_of::<LinuxDriverDescriptor>() as u16,
        payload_size: 4096,
        checksum: 1,
    };
    fn candidate(hash: u64) -> LinuxCandidate {
        LinuxCandidate {
            descriptor: LinuxDriverDescriptor {
                vendor: 0x8086,
                device: 0x100e,
                class: 0x0200,
                api_version: 6,
                module_hash: hash,
                signed: true,
            },
            priority: 10,
        }
    }
    #[test]
    fn accepts_dependency_chain() {
        let candidates = [candidate(1), candidate(2), candidate(3)];
        let deps = [
            Dependency {
                driver_hash: 1,
                required_hash: 2,
            },
            Dependency {
                driver_hash: 2,
                required_hash: 3,
            },
        ];
        assert!(
            prepare_graph_guarded(DEV, &candidates, &deps, &LinuxRuntime::new(6), HDR, RES).is_ok()
        )
    }
    #[test]
    fn rejects_cycle_before_prepare() {
        let candidates = [candidate(1), candidate(2), candidate(3)];
        let deps = [
            Dependency {
                driver_hash: 1,
                required_hash: 2,
            },
            Dependency {
                driver_hash: 2,
                required_hash: 3,
            },
            Dependency {
                driver_hash: 3,
                required_hash: 1,
            },
        ];
        assert_eq!(
            prepare_graph_guarded(DEV, &candidates, &deps, &LinuxRuntime::new(6), HDR, RES),
            Err(GraphTransactionError::Graph(GraphError::Cycle))
        )
    }
}

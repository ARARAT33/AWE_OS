#![no_std]

/// Stable CellKernel/service contract.
///
/// 60.2 freezes the minimal kernel-to-service boundary. The kernel exposes
/// capabilities and IPC primitives; policy and service implementations stay
/// outside the kernel.
pub const ABI_MAJOR: u16 = 1;
pub const ABI_MINOR: u16 = 2;
pub const KERNEL_NAME: &[u8] = b"CellKernel";
pub const OS_NAME: &[u8] = b"AWE_OS";

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelCapability {
    Memory = 0,
    Interrupts = 1,
    Scheduling = 2,
    Processes = 3,
    Ipc = 4,
    Syscalls = 5,
    Security = 6,
    DeviceGrant = 7,
    Dma = 8,
    SharedMemory = 9,
}

impl KernelCapability { pub const fn bit(self) -> u64 { 1u64 << (self as u8) } }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilitySet(u64);

impl CapabilitySet {
    pub const EMPTY: Self = Self(0);
    pub const fn from_bits(bits: u64) -> Self { Self(bits) }
    pub const fn bits(self) -> u64 { self.0 }
    pub const fn contains(self, capability: KernelCapability) -> bool { self.0 & capability.bit() != 0 }
    pub const fn with(self, capability: KernelCapability) -> Self { Self(self.0 | capability.bit()) }
    pub const fn without(self, capability: KernelCapability) -> Self { Self(self.0 & !capability.bit()) }
}

pub const REQUIRED_RUNTIME_CAPABILITIES: CapabilitySet = CapabilitySet::EMPTY
    .with(KernelCapability::Memory)
    .with(KernelCapability::Interrupts)
    .with(KernelCapability::Scheduling)
    .with(KernelCapability::Processes)
    .with(KernelCapability::Ipc)
    .with(KernelCapability::Syscalls)
    .with(KernelCapability::Security);

#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceId {
    Driverd = 1,
    Appd = 2,
    Asappd = 3,
    Ayuid = 4,
    Aweterminald = 5,
    Awebusd = 6,
    Aweupdated = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceContract {
    pub service: ServiceId,
    pub abi_major: u16,
    pub abi_minor: u16,
    pub required_capabilities: CapabilitySet,
}

impl ServiceContract {
    pub const fn new(service: ServiceId, abi_major: u16, abi_minor: u16, required_capabilities: CapabilitySet) -> Self {
        Self { service, abi_major, abi_minor, required_capabilities }
    }

    pub const fn accepts_kernel(self, kernel: KernelContract) -> bool {
        kernel.abi_major == self.abi_major
            && kernel.abi_minor >= self.abi_minor
            && kernel.capabilities.bits() & self.required_capabilities.bits() == self.required_capabilities.bits()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContractError { MajorVersionMismatch, MinorVersionTooOld, MissingCapability }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelContract {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub capabilities: CapabilitySet,
}

impl KernelContract {
    pub const fn current(capabilities: CapabilitySet) -> Self { Self { abi_major: ABI_MAJOR, abi_minor: ABI_MINOR, capabilities } }
    pub const fn is_compatible_with(self, required_major: u16) -> bool { self.abi_major == required_major }
    pub const fn has_runtime_baseline(self) -> bool {
        self.capabilities.bits() & REQUIRED_RUNTIME_CAPABILITIES.bits() == REQUIRED_RUNTIME_CAPABILITIES.bits()
    }
    pub const fn validate_service(self, service: ServiceContract) -> Result<(), ContractError> {
        if self.abi_major != service.abi_major { return Err(ContractError::MajorVersionMismatch); }
        if self.abi_minor < service.abi_minor { return Err(ContractError::MinorVersionTooOld); }
        if self.capabilities.bits() & service.required_capabilities.bits() != service.required_capabilities.bits() {
            return Err(ContractError::MissingCapability);
        }
        Ok(())
    }
}

const USERSPACE_SERVICE_CAPS: CapabilitySet = CapabilitySet::EMPTY
    .with(KernelCapability::Ipc)
    .with(KernelCapability::Security);

pub const DRIVERD_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Driverd, ABI_MAJOR, ABI_MINOR,
    USERSPACE_SERVICE_CAPS
        .with(KernelCapability::DeviceGrant)
        .with(KernelCapability::Dma)
        .with(KernelCapability::SharedMemory),
);

pub const APPD_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Appd, ABI_MAJOR, ABI_MINOR,
    USERSPACE_SERVICE_CAPS.with(KernelCapability::SharedMemory),
);

pub const ASAPPD_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Asappd, ABI_MAJOR, ABI_MINOR, USERSPACE_SERVICE_CAPS,
);

pub const AYUID_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Ayuid, ABI_MAJOR, ABI_MINOR,
    USERSPACE_SERVICE_CAPS.with(KernelCapability::SharedMemory),
);

pub const AWETERMINALD_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Aweterminald, ABI_MAJOR, ABI_MINOR, USERSPACE_SERVICE_CAPS,
);

pub const AWEBUSD_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Awebusd, ABI_MAJOR, ABI_MINOR, USERSPACE_SERVICE_CAPS,
);

pub const AWEUPDATED_CONTRACT: ServiceContract = ServiceContract::new(
    ServiceId::Aweupdated, ABI_MAJOR, ABI_MINOR,
    USERSPACE_SERVICE_CAPS.with(KernelCapability::SharedMemory),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_operations_are_deterministic() {
        let set = CapabilitySet::EMPTY.with(KernelCapability::Memory).with(KernelCapability::Ipc);
        assert!(set.contains(KernelCapability::Memory));
        assert!(set.contains(KernelCapability::Ipc));
        assert!(!set.contains(KernelCapability::Dma));
        assert!(!set.without(KernelCapability::Memory).contains(KernelCapability::Memory));
    }

    #[test]
    fn current_contract_requires_declared_baseline() {
        let contract = KernelContract::current(REQUIRED_RUNTIME_CAPABILITIES);
        assert!(contract.is_compatible_with(ABI_MAJOR));
        assert!(contract.has_runtime_baseline());
        assert!(!contract.is_compatible_with(ABI_MAJOR + 1));
    }

    #[test]
    fn every_canonical_service_has_a_contract() {
        let contracts = [
            DRIVERD_CONTRACT,
            APPD_CONTRACT,
            ASAPPD_CONTRACT,
            AYUID_CONTRACT,
            AWETERMINALD_CONTRACT,
            AWEBUSD_CONTRACT,
            AWEUPDATED_CONTRACT,
        ];
        assert_eq!(contracts.len(), 7);
        assert_eq!(contracts[0].service, ServiceId::Driverd);
        assert_eq!(contracts[6].service, ServiceId::Aweupdated);
    }

    #[test]
    fn service_contract_checks_version_and_capabilities() {
        let kernel = KernelContract::current(
            REQUIRED_RUNTIME_CAPABILITIES
                .with(KernelCapability::DeviceGrant)
                .with(KernelCapability::Dma)
                .with(KernelCapability::SharedMemory),
        );
        assert_eq!(kernel.validate_service(DRIVERD_CONTRACT), Ok(()));
        let weak = KernelContract::current(REQUIRED_RUNTIME_CAPABILITIES);
        assert_eq!(weak.validate_service(DRIVERD_CONTRACT), Err(ContractError::MissingCapability));
    }

    #[test]
    fn minor_versions_are_forward_compatible() {
        let kernel = KernelContract::current(REQUIRED_RUNTIME_CAPABILITIES);
        let older = ServiceContract::new(ServiceId::Appd, ABI_MAJOR, 0, CapabilitySet::EMPTY.with(KernelCapability::Ipc));
        assert_eq!(kernel.validate_service(older), Ok(()));
    }
}

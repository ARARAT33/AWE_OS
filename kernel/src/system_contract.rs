#![no_std]

/// Stable identity and capability metadata exposed to kernel integration code.
///
/// This is deliberately data-only: architecture code and policy engines remain
/// responsible for enforcing the individual contracts.
pub const ABI_MAJOR: u16 = 1;
pub const ABI_MINOR: u16 = 0;
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
    Drivers = 6,
    Storage = 7,
    Network = 8,
    Security = 9,
}

impl KernelCapability {
    pub const fn bit(self) -> u64 {
        1u64 << (self as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilitySet(u64);

impl CapabilitySet {
    pub const EMPTY: Self = Self(0);

    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub const fn contains(self, capability: KernelCapability) -> bool {
        self.0 & capability.bit() != 0
    }

    pub const fn with(self, capability: KernelCapability) -> Self {
        Self(self.0 | capability.bit())
    }

    pub const fn without(self, capability: KernelCapability) -> Self {
        Self(self.0 & !capability.bit())
    }
}

/// Minimum capability set expected before the kernel can report a fully
/// initialized execution environment.
pub const REQUIRED_RUNTIME_CAPABILITIES: CapabilitySet = CapabilitySet::EMPTY
    .with(KernelCapability::Memory)
    .with(KernelCapability::Interrupts)
    .with(KernelCapability::Scheduling)
    .with(KernelCapability::Processes)
    .with(KernelCapability::Ipc)
    .with(KernelCapability::Syscalls)
    .with(KernelCapability::Security);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelContract {
    pub abi_major: u16,
    pub abi_minor: u16,
    pub capabilities: CapabilitySet,
}

impl KernelContract {
    pub const fn current(capabilities: CapabilitySet) -> Self {
        Self {
            abi_major: ABI_MAJOR,
            abi_minor: ABI_MINOR,
            capabilities,
        }
    }

    pub const fn is_compatible_with(self, required_major: u16) -> bool {
        self.abi_major == required_major
    }

    pub const fn has_runtime_baseline(self) -> bool {
        self.capabilities.bits() & REQUIRED_RUNTIME_CAPABILITIES.bits()
            == REQUIRED_RUNTIME_CAPABILITIES.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_operations_are_deterministic() {
        let set = CapabilitySet::EMPTY
            .with(KernelCapability::Memory)
            .with(KernelCapability::Ipc);
        assert!(set.contains(KernelCapability::Memory));
        assert!(set.contains(KernelCapability::Ipc));
        assert!(!set.contains(KernelCapability::Network));
        assert!(!set.without(KernelCapability::Memory).contains(KernelCapability::Memory));
    }

    #[test]
    fn current_contract_requires_only_declared_baseline() {
        let contract = KernelContract::current(REQUIRED_RUNTIME_CAPABILITIES);
        assert!(contract.is_compatible_with(ABI_MAJOR));
        assert!(contract.has_runtime_baseline());
        assert!(!contract.is_compatible_with(ABI_MAJOR + 1));
    }
}

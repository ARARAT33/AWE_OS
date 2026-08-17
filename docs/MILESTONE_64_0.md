# AWE_OS 64.0 — Hardware Access Boundary Freeze

## Status

**Milestone: 64.0 — code gate complete.**

64.0 closes the remaining kernel/device-boundary preparation work immediately before the 65% hardware execution checkpoint. It does not claim real PCI, ACPI, APIC/IOAPIC, VirtIO, DMA/IOMMU or QEMU hardware execution.

## Completed

- [x] Overflow-safe MMIO region contract.
- [x] Overflow-safe PIO region contract.
- [x] Explicit MMIO/PIO access kind.
- [x] Bounded register-window containment checks.
- [x] Explicit interrupt ownership contract.
- [x] Line/MSI/MSI-X interrupt mode model.
- [x] Explicit power-state model and bounded transition policy.
- [x] Canonical kernel `DeviceAccessContract` ties all access resources to one device identity.
- [x] Driver-service `HardwareAccessPlan` mirrors the same ownership rule on the isolated driver side.
- [x] Driver hardware access contracts are exported by `driverd` without moving driver implementation into CellKernel.
- [x] Unit coverage for overflow, bounds, interrupt ownership, power transitions and identity consistency.

## Architecture

```text
CellKernel
  |
  +-- Device identity
  +-- Resource/capability boundary
  +-- MMIO/PIO contract
  +-- Interrupt ownership contract
  +-- Power-state policy
  |
  +----------------------+
                         |
                       driverd
                         +-- HardwareAccessPlan
                         +-- Driver lifecycle
                         +-- Dependency/resource ownership
                         +-- Health/recovery contracts
```

Concrete discovery/programming is deliberately outside this gate.

## Reserved for 65% checkpoint

- PCI/PCIe enumeration and BAR discovery;
- ACPI discovery and power-resource execution;
- APIC/IOAPIC implementation and real interrupt routing;
- VirtIO transport and queue execution;
- real DMA/IOMMU enforcement;
- concrete storage/network/input/display drivers;
- QEMU hardware certification.

## Acceptance criteria

- [x] MMIO and PIO regions reject zero-length and overflowing ranges.
- [x] Sub-range access is bounded by the declared region.
- [x] Interrupt ownership requires a valid vector and explicit mode.
- [x] Power transitions are deterministic and reject unsupported jumps.
- [x] Device-side access contracts cannot mix resource ownership across device IDs.
- [x] Driver-side access plans cannot mix resources or interrupts across driver IDs.
- [x] No concrete hardware execution is counted toward 64%.

## Next gate

**65% checkpoint:** execute and validate the real hardware path together — PCI/ACPI/APIC/IOAPIC/VirtIO plus the first concrete driver families and emulator validation.

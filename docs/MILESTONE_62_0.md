# AWE_OS 62.0 — Driver Resource/Capability Integration

## Status

**Milestone: 62.0 — code gate complete.**

62.0 is the next sub-gate after the 61.0 Architecture Freeze. It follows the master plan by preparing the standalone driver service for real hardware execution without prematurely counting the 65% PCI/ACPI/VirtIO checkpoint.

## Scope completed

- [x] Driver grants bind a driver service to a stable capability endpoint.
- [x] Driver grants bind to one canonical device identity.
- [x] MMIO, I/O, DMA and interrupt budgets remain bounded.
- [x] Capability permission is checked independently from resource budget.
- [x] Service, endpoint and device mismatches fail closed.
- [x] Driver grants are fixed-layout and allocation-free.
- [x] Existing exact/class/fallback matching remains deterministic.
- [x] Unit coverage covers service mismatch, endpoint validity, device identity, capability denial and resource exhaustion.

## Architecture boundary

```text
CellKernel
  |
  +-- Service/Capability endpoint
  |
  +-- Device identity
  |
  +-- Bounded resource grant
  |
  +-- Admission validation
  |
  +--> driverd
          |
          +--> concrete discovery/probe/bind/execute
```

CellKernel still does not enumerate PCI, parse ACPI, program VirtIO, perform DMA, or execute driver code. Those remain in `services/driverd` and the 65% validation checkpoint.

## Acceptance criteria

- [x] A driver grant cannot be used by the wrong service.
- [x] A driver grant cannot be used with an invalid/wrong endpoint.
- [x] A driver grant cannot be used for another device.
- [x] A driver grant cannot request resources above its allowed budget.
- [x] A driver grant cannot exercise a capability it was not given.
- [x] The existing device registry and matching model remain bounded.
- [x] No 65% PCI/ACPI/VirtIO implementation is counted here.

## Deliberately deferred to 65%

- PCI/PCIe enumeration
- ACPI discovery
- APIC/IOAPIC execution
- VirtIO transport
- concrete VirtIO block/network/input/display drivers
- real DMA/IOMMU enforcement
- QEMU hardware certification

## Progress accounting

The 62% label represents this validated engineering sub-gate only. It does not claim that the 65% driver checkpoint has been completed, and the global percentage must not advance again until the next master-plan gate is actually implemented and validated.

# AWE_OS 63.0 — Driver Dependency, Ownership and Health Gate

## Status

**Milestone: 63.0 — code gate complete.**

63.0 advances the 61.6–64.x driver-service preparation block from the master plan. It adds the deterministic contracts needed before concrete PCI/ACPI/VirtIO execution, without counting the 65% hardware checkpoint early.

## Completed

- [x] Bounded driver dependency graph.
- [x] Self-dependency rejection.
- [x] Simple transitive cycle rejection.
- [x] Explicit per-driver resource ownership record.
- [x] MMIO/I/O/DMA/interrupt ownership remains bounded.
- [x] Driver health state with consecutive-failure tracking.
- [x] Bounded restart accounting.
- [x] Deterministic health recovery transition.
- [x] All primitives are fixed-capacity/allocation-free.
- [x] Driver-service exports expose the new contracts.
- [x] Unit tests cover dependency cycles, resource ownership and restart behavior.

## Architecture

```text
CellKernel
   |
   +-- capability endpoint
   +-- device/resource grant
   |
   v
 driverd
   +-- manifest/lifecycle
   +-- dependency graph
   +-- resource ownership
   +-- health monitor contract
   +-- restart/quarantine policy
```

Concrete PCI/PCIe enumeration, ACPI discovery, APIC/IOAPIC execution, VirtIO transport, DMA/IOMMU hardware enforcement and QEMU device certification remain reserved for the 65% checkpoint.

## Acceptance rule

63% is earned only for the implemented driver-service preparation contract above. No UI, application, filesystem, networking or compatibility work is counted toward this percentage.

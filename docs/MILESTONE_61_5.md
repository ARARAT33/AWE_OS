# AWE_OS 61.5 — Canonical Device Model Freeze

## Status

**Milestone: 61.5 — code gate complete.**

61.5 is the first execution-side hardware sub-gate after the 60.0–61.0 Architecture Freeze. It prepares the exact device/driver boundary without counting the later 65% PCI/ACPI/VirtIO implementation.

## Scope completed

- [x] Hardware-neutral `DeviceId`/`DeviceClass`/`DeviceState` model remains stable.
- [x] Canonical exact/class/fallback driver matching contract.
- [x] Explicit bounded resource grants for MMIO, I/O, DMA and interrupt ownership.
- [x] Fail-closed binding decision when identity or resource budget does not match.
- [x] Device contracts expose canonical exact-match descriptors.
- [x] All new primitives are fixed-layout/allocation-free.
- [x] Unit coverage for strict matching and resource ownership.

## Intentionally not counted in 61.5

The following remain reserved for the 65% checkpoint and later driver milestones:

- PCI/PCIe enumeration;
- ACPI discovery;
- APIC/IOAPIC implementation;
- VirtIO transport;
- concrete hardware driver execution;
- DMA/IOMMU hardware enforcement;
- QEMU hardware certification.

## Architecture rule

CellKernel owns only the device identity/resource contract. Concrete discovery, probing, binding execution, interrupts, DMA programming and driver lifecycle remain in `services/driverd` and later hardware milestones.

## Acceptance criteria

- [x] `DeviceMatch` is deterministic and exact matching is strict.
- [x] Class matching is available for intentionally generic drivers.
- [x] Resource grants are bounded and include a device identity.
- [x] Binding rejects mismatched identity or over-budget resources.
- [x] Existing bounded device registry remains intact.
- [x] No PCI/VirtIO implementation is introduced early.

## Next gate

**61.6–64.x:** driver-service preparation, resource/capability integration and the remaining kernel-side device boundary work.

**65% checkpoint:** the first large validation point, where real PCI/ACPI/VirtIO/driver execution is expected to be implemented and validated together.

# AWE_OS 62.5 — Native Driver Manifest & Lifecycle Freeze

## Status

**Milestone: 62.5 — code gate complete.**

62.5 implements the next sub-gate after the device/resource boundary: a stable native driver-service manifest and deterministic lifecycle contract. It does **not** claim concrete PCI/ACPI/VirtIO execution; those remain reserved for the 65% checkpoint.

## Completed

- [x] Driver manifest with stable identity, class and ABI version.
- [x] Architecture target mask.
- [x] Explicit declared capability mask.
- [x] Trust state with fail-closed execution admission.
- [x] Canonical lifecycle: discover → identify → probe → bind → initialize → run → suspend/resume → stop → remove/recover.
- [x] Invalid lifecycle transitions are rejected.
- [x] Driver registry now carries ABI minor version and trust metadata.
- [x] Unverified driver descriptors are rejected by registration.
- [x] Built-in driver catalog migrated to the new metadata shape.
- [x] Manifest export from registered driver descriptors.
- [x] Unit coverage for lifecycle, trust, manifest and registry invariants.

## Boundary rule

`driverd` owns the concrete driver implementation and lifecycle supervisor in user space. CellKernel owns only the capability/resource/device contract required to isolate and communicate with that service.

## Not counted before 65%

- PCI/PCIe enumeration;
- ACPI discovery;
- APIC/IOAPIC hardware implementation;
- VirtIO transport and real VirtIO drivers;
- hardware DMA/IOMMU enforcement;
- QEMU hardware certification.

## Acceptance criteria

- [x] A driver cannot enter the execution lifecycle unless its trust state is verified.
- [x] Lifecycle transitions are deterministic and bounded.
- [x] Driver manifests carry architecture and capability declarations.
- [x] Registry metadata is consistent with the manifest contract.
- [x] Legacy built-in driver metadata remains compatible with the frozen ABI.

## Next gate

**62.6–64.x:** complete the remaining driver-service preparation, including dependency/resource graphs and suspend/recovery policy. The **65% checkpoint** is the first combined hardware-validation milestone for real PCI/ACPI/VirtIO/driver execution.

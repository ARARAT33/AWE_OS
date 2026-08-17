# AWE_OS 65.0 — Hardware Execution Checkpoint

## Status

**Milestone: 65.0 — implementation checkpoint completed; validation is required for release-grade certification.**

This is the first large hardware checkpoint in the 60→100 master plan. It turns the previously frozen device/driver contracts into a concrete reference platform inside the standalone `driverd` service plane.

## Completed implementation

### PCI
- [x] Bounded PCI configuration-space enumerator.
- [x] Bus/device/function traversal.
- [x] Multi-function device detection.
- [x] Vendor/device/class/subclass/prog-if/header/interrupt extraction.
- [x] x86_64 PCI Configuration Mechanism #1 backend (`CF8/CFC`).
- [x] Deterministic table capacity/backpressure.

### ACPI
- [x] RSDP validation for legacy and revision 2+ layouts.
- [x] Primary and extended checksum validation.
- [x] Root pointer/table-directory parsing primitives.
- [x] SDT header bounds/checksum validation.
- [x] Table lookup.
- [x] MADT base-address/flags parsing primitive.

### APIC/IOAPIC
- [x] Local APIC state model.
- [x] IOAPIC GSI ownership model.
- [x] IRQ vector validation.
- [x] Mask/unmask route model.
- [x] Overflow-safe GSI range handling.

### VirtIO 1.x
- [x] Version-1 feature requirement.
- [x] Device status progression.
- [x] Feature negotiation.
- [x] Driver-OK readiness gate.
- [x] Fixed-capacity queue model.
- [x] Queue size/power-of-two validation.

### Reference VirtIO devices
- [x] Block request validation.
- [x] Network frame contract.
- [x] Input event contract.
- [x] Display rectangle validation.
- [x] Typed block/network/input/GPU reference device handles.

### Driver catalog
- [x] PCI/ACPI/APIC entries.
- [x] VirtIO transport entry.
- [x] VirtIO block/network/input/GPU entries.
- [x] Display/input/audio entries.
- [x] AHCI/NVMe metadata entries.
- [x] Linux/Windows/Android compatibility metadata remains separated from native hardware drivers.

## Architecture rule

All concrete hardware discovery and driver operations remain in `services/driverd`. `CellKernel` continues to expose only the capability, IPC, process and device/resource contract surface.

## Validation requirements

The code gate alone does not mean every real-hardware gate has passed. The remaining certification evidence for this checkpoint is:

- QEMU boot with PCI and VirtIO devices;
- workspace formatting/check/tests/Clippy;
- QEMU storage/network/input/display exercise;
- hardware access fault and bounded-resource tests;
- no critical unresolved build or runtime blocker.

The repository's quality workflow is configured to run on pushes to `main`; this checkpoint must be considered release-certified only when those automated results are green.

## Deliberately not included in 65.0

- Native filesystem completion.
- Full TCP/IP stack.
- Userspace loader/init completion.
- AWOSA runtime.
- `.awos` package system.
- `.asd` package tooling.
- AYUI desktop.
- Compatibility application runtimes.
- ASAPP/App Builder.
- AWEUpdate/live update.

Those belong to later stages in the master execution plan.

# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**65% — AWE_OS 1.0 implementation readiness.**

The 65% hardware execution checkpoint is now implemented in the standalone driver plane. Automated CI/QEMU certification is a separate evidence gate and must be green before this percentage is called release-certified.

## Implemented kernel foundations

- Intent-carrying authorization with required rights, impact class and resource budget.
- Deterministic security policy engine with explicit allow/deny reasons.
- Bounded, allocation-free causal provenance journal.
- Non-escalating capability derivation and explicit revocation state.
- Process states and consumable CPU, memory and IPC budgets.
- Allocation-free token-bucket rate limiter.
- Bounded scheduler queue and fixed-priority scheduling primitive.
- Saturating deterministic scheduler tick clock.
- Deterministic allocation-free scheduler dispatcher with explicit current-process rotation and yield semantics.
- Typed physical/virtual addresses with overflow-safe alignment.
- Deterministic early page mapper with duplicate-map rejection and fail-closed validation.
- Bounded early boot identity mapper with overflow and capacity rejection.
- x86_64 four-level page-table representation and mapping validation.
- x86_64 bootstrap identity mapping of the first 1 GiB using 2 MiB pages.
- Real x86_64 CR3 activation from the CellKernel bootstrap path after boot-protocol validation.
- x86_64 IDT gate encoding and safe early timer-vector installation primitive.
- x86_64 CR3/RFLAGS/I/O primitives.
- Monotonic boot phases and terminal failure state.
- Hardware driver contracts for MMIO, DMA, interrupt mode and device identity.
- Bounded device-bus registry.
- VirtIO feature negotiation primitive requiring VirtIO 1.x before activation.
- Validated syscall dispatch gate with argument/error validation and resource-budget enforcement.
- AWE Capsule and XenoSense security foundations.
- Cloud CI security/release gates.
- **60.2 service-contract freeze:** CellKernel ABI 1.2, typed service IDs, explicit capability admission, typed IPC channels/opcodes, and contracts for the full canonical service roster.
- **60.5 service/process freeze:** explicit service-to-process ownership, service classes, lifecycle states, bounded CPU/memory/IPC budgets, fixed-capacity service registry, canonical seven-service roster, and fail-closed capability admission.
- **60.6–61.0 transport freeze:** capability handles, service endpoints, versioned HELLO handshake, bounded shared-memory-style rings, async request tracking, event queues, canonical service/channel mapping, and frozen IPC opcode validation.
- **61.5 device-boundary freeze:** canonical device matching, explicit resource grants, bounded MMIO/I/O/DMA/interrupt ownership accounting, and fail-closed driver binding decisions.
- **62.0 driver capability integration:** driver grants bind a driver service to an authenticated capability endpoint and one canonical device identity while independently enforcing capability and resource budgets.
- **62.5 native driver manifest/lifecycle freeze:** ABI-aware driver manifests, architecture/capability declarations, verified-trust admission, deterministic lifecycle transitions, and registry-level trust/ABI metadata.
- **63.0 driver dependency/ownership/health integration:** bounded dependency graph, cycle rejection, per-driver resource ownership, failure counters, bounded restart accounting, and deterministic health recovery state.
- **64.0 hardware access boundary freeze:** overflow-safe MMIO/PIO regions, bounded sub-range checks, explicit interrupt ownership modes, deterministic power-state transitions, and device-consistent access plans in both CellKernel and driverd.
- **65.0 hardware execution checkpoint:** PCI enumeration primitives, x86 PCI config-space backend, ACPI/RSDP/table validation, APIC/IOAPIC routing model, VirtIO 1.x transport/queue state machine, and typed block/network/input/display reference devices.

## 65.0 milestone — IMPLEMENTATION COMPLETE

The 65% hardware execution checkpoint is implemented in `services/driverd` without moving concrete driver code into CellKernel:

### PCI
- [x] Bounded bus/device/function enumeration.
- [x] Multi-function detection.
- [x] Vendor/device/class/subclass/prog-if/header/interrupt extraction.
- [x] x86_64 Configuration Mechanism #1 backend (`CF8/CFC`).
- [x] Fixed-capacity discovery table.

### ACPI
- [x] RSDP legacy/revision-2+ validation.
- [x] Primary/extended checksum validation.
- [x] Root table pointer parsing.
- [x] SDT length/checksum bounds validation.
- [x] Table lookup.
- [x] MADT base-address/flags parsing.

### APIC/IOAPIC
- [x] Local APIC state model.
- [x] IOAPIC GSI ownership model.
- [x] IRQ vector validation.
- [x] Mask/unmask routing model.
- [x] Overflow-safe GSI range handling.

### VirtIO
- [x] VirtIO 1.x feature requirement.
- [x] Device status progression.
- [x] Feature negotiation.
- [x] Driver-OK readiness gate.
- [x] Bounded power-of-two queue validation.

### Reference devices
- [x] VirtIO block request/range validation.
- [x] Network frame contract.
- [x] Input event contract.
- [x] Display rectangle validation.
- [x] Typed Block/Network/Input/GPU reference device descriptors.

### Driver ecosystem
- [x] Canonical PCI/ACPI/APIC/VirtIO metadata.
- [x] VirtIO block/network/input/GPU metadata.
- [x] AHCI/NVMe metadata.
- [x] Linux/Windows/Android compatibility metadata remains separate from native hardware drivers.

## 65.0 validation evidence gate — PENDING

Implementation completion is not the same as release certification. These evidence items are intentionally separate:

- [ ] GitHub Quality Gate finishes green on the 65% checkpoint tree.
- [ ] Boot Image workflow finishes green.
- [ ] QEMU PCI/VirtIO device exercise passes.
- [ ] Storage/network/input/display runtime exercise passes.
- [ ] No high/critical build or runtime blocker remains.

The current repository workflows are configured to run automatically on pushes to `main`. The checkpoint is **implemented**, while certification remains evidence-driven.

## Driver roadmap after 65%

### Completed at the implementation checkpoint
- [x] PCI discovery contract and x86 backend.
- [x] ACPI discovery primitives.
- [x] APIC/IOAPIC routing model.
- [x] VirtIO transport/queue model.
- [x] VirtIO block/network/input/display reference protocols.

### Still ahead
- [ ] Real DMA/IOMMU hardware enforcement.
- [ ] Full NVMe/AHCI runtime execution.
- [ ] xHCI/USB runtime.
- [ ] HID runtime.
- [ ] Ethernet/Wi-Fi/Bluetooth runtime.
- [ ] Audio runtime.
- [ ] GPU acceleration runtime.
- [ ] Hardware certification matrix.

## Later stages

The remaining 65→100 roadmap continues exactly as defined in `docs/AWE_OS_100_PERCENT_MASTER_PLAN.md`: storage/filesystem → networking → userspace/services → AWOSA → `.awos` → compatibility → AYUI → App Builder → native ecosystem → security/update/recovery → multi-architecture → release validation.

## Definition of done

AWE_OS is considered fully product-ready only when the required boot, kernel, driver, security, recovery and release gates are implemented and their automated CI/QEMU validation is green. Documentation or architectural placeholders alone do not satisfy a gate.

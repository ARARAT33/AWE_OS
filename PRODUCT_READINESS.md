# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**60.5% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage follows the AWE_OS 60–100 master plan and advances only when the corresponding implementation gate is completed. Documentation alone never advances the percentage. The 60.5% gate specifically represents completion of the process/service ownership freeze inside the 60–61% Architecture Freeze block.

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
- **60.5 service/process freeze:** explicit service-to-process ownership, service classes, lifecycle states, bounded CPU/memory/IPC budgets, a fixed-capacity service registry, canonical seven-service roster, and fail-closed capability admission.

## 60.2 milestone — COMPLETE

The 60.2 gate freezes the minimal kernel-to-service boundary:

- [x] CellKernel ABI 1.2.
- [x] Stable service identifiers for driverd/appd/ASAPP/AYUI/terminal/bus/update services.
- [x] Explicit capability requirements for service admission.
- [x] Fail-closed major/minor ABI validation.
- [x] Forward-compatible minor-version validation.
- [x] Typed IPC service channels.
- [x] Stable IPC opcodes for hello/ping/start/stop/reset/query/event/handoff.
- [x] driverd ABI 1.2 alignment.
- [x] appd ABI 1.2 alignment.
- [x] Unit coverage for capability, version and IPC invariants.

The detailed specification is `docs/MILESTONE_60_2.md`.

## 60.5 milestone — COMPLETE

The 60.5 gate freezes the process/service ownership model for the complete platform boundary without moving implementations into CellKernel:

- [x] `ServiceDescriptor` with stable `ServiceId` and owning `ProcessId`.
- [x] Explicit service classes: system, hardware, application, interface, compatibility and update.
- [x] Explicit lifecycle: declared, starting, running, stopping, failed, quarantined.
- [x] Per-service CPU, memory and IPC budgets.
- [x] Capability set attached to each service process.
- [x] Fail-closed capability admission before startup.
- [x] Allocation-free fixed-layout service descriptors.
- [x] Bounded duplicate-safe `ServiceRegistry<N>`.
- [x] Canonical seven-service roster frozen.
- [x] All seven canonical services mapped to versioned service contracts.
- [x] Unit tests for lifecycle, registry bounds/duplicates, roster stability and capability rejection.
- [x] CellKernel remains driver/application implementation free.

The detailed specification is `docs/MILESTONE_60_5.md`.

## Hardware and driver intelligence

AWE_OS targets real hardware, virtual machines and cloud/server environments through one capability-controlled Driver HAL. Foreign kernel ABIs are not executed directly inside CellKernel.

Implemented foundations include cross-OS provenance manifests, verified-only binding, bounded compatibility registry, offline/online driver database architecture, staged authenticated updates, rollback-safe updates, deterministic driver experience tracking and fail-closed behavior for unknown/unverified hardware.

## Driver roadmap

### Virtualization
- [x] Driver HAL and device contracts.
- [x] VirtIO feature negotiation foundation.
- [x] Compatibility manifests and bounded learning state.
- [ ] VirtIO PCI transport.
- [ ] VirtIO block driver.
- [ ] VirtIO network driver.
- [ ] VirtIO console/entropy/input drivers.
- [ ] VirtIO GPU driver.
- [ ] Automated QEMU end-to-end device tests.

### PC/server hardware
- [ ] PCI enumeration.
- [ ] ACPI tables and power-management.
- [ ] APIC/IOAPIC.
- [ ] NVMe/AHCI.
- [ ] xHCI/USB.
- [ ] HID keyboard/mouse.
- [ ] Ethernet.
- [ ] Wi-Fi.
- [ ] Audio.
- [ ] GPU/display.
- [ ] IOMMU/SMMU DMA isolation.

### Cross-OS coverage
- [x] Driver source/provenance model.
- [x] Offline/online database architecture.
- [x] Deterministic driver experience tracking.
- [ ] Linux hardware-ID ingestion implementation.
- [ ] Android vendor compatibility adapters.
- [ ] Windows documented-device compatibility adapters.
- [ ] BSD compatibility adapters.
- [ ] Hardware certification matrix.

## Bootloader

- [x] AWE image identity and architecture validation contract.
- [x] Loader-owned memory validation contract.
- [x] Monotonic boot phases and terminal failure state.
- [x] Kernel-side x86_64 bootstrap page-table construction and CR3 activation.
- [ ] Complete x86_64 UEFI boot implementation.
- [ ] Loader-owned page-table handoff and CR3 activation.
- [ ] Signed/measured release image verification in the boot path.
- [ ] Automated QEMU boot certification.

## 60.5 milestone boundary

The 60.5 milestone represents a genuine architecture-freeze gate: CellKernel has a bounded process model and a fixed service ownership contract; every canonical system service has a stable identity, process owner, lifecycle, resource budget and capability requirements; driver/application/UI/update implementations remain in their user-space service planes. The next planned block is 60.6–60.8 execution-side IPC/service transport.

## Definition of done

AWE_OS is considered product-ready only when the required boot, kernel, driver, security, recovery and release gates are implemented and their automated CI/QEMU validation is green. Documentation or architectural placeholders alone do not satisfy a gate.

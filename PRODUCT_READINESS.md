# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**61.5% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage follows the AWE_OS 60–100 master plan and advances only when the corresponding implementation gate is completed. Documentation alone never advances the percentage. The 61.5% gate is a hardware-boundary sub-gate between the completed 61.0 Architecture Freeze and the 65% driver execution checkpoint.

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

## 61.5 milestone — COMPLETE

The 61.5 gate is deliberately hardware-neutral. It completes the common device/driver contract needed before concrete hardware discovery and execution:

- [x] Stable `DeviceId`, `DeviceClass` and `DeviceState` model.
- [x] Exact, class and fallback driver matching semantics.
- [x] Canonical device-to-driver match descriptors.
- [x] Explicit resource grant contract for MMIO, I/O, DMA and interrupts.
- [x] Resource ownership is bounded by a device-specific budget.
- [x] Binding fails closed on identity mismatch or resource over-allocation.
- [x] Existing bounded device registry remains intact.
- [x] New primitives are fixed-layout and allocation-free.
- [x] Unit coverage for strict matching and resource ownership.

## 61.5 boundary — NOT COUNTED YET

The following remain intentionally reserved for the 65% checkpoint and later production gates:

- PCI/PCIe enumeration;
- ACPI discovery;
- APIC/IOAPIC implementation;
- VirtIO transport and concrete drivers;
- real hardware driver execution;
- hardware DMA/IOMMU enforcement;
- QEMU device certification.

## Driver roadmap

### Virtualization
- [x] Driver HAL and device contracts.
- [x] VirtIO feature negotiation foundation.
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
- [x] Offline/online driver database architecture.
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

## 61.5 milestone boundary

The 61.5 milestone represents completion of the hardware-neutral device boundary required by the master execution order: the kernel can describe devices, define canonical driver matching and account for bounded resource ownership, while concrete PCI/ACPI/VirtIO discovery and driver execution remain outside this gate. No 65% checkpoint work is counted early.

## Definition of done

AWE_OS is considered product-ready only when the required boot, kernel, driver, security, recovery and release gates are implemented and their automated validation is green. Documentation or architectural placeholders alone do not satisfy a gate.

# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**64% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage follows the AWE_OS 60–100 master plan and advances only when the corresponding implementation gate is completed. Documentation alone never advances the percentage. The 64% gate closes the remaining hardware-boundary preparation inside the 61.6–64.x block; the 65% hardware execution checkpoint remains separate.

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

## 62.0 milestone — COMPLETE

The 62.0 gate prepares the standalone driver service for real hardware execution without counting the 65% PCI/ACPI/VirtIO implementation.

## 62.5 milestone — COMPLETE

The 62.5 gate freezes the native driver-service contract required immediately before concrete hardware execution.

## 63.0 milestone — COMPLETE

The 63.0 gate advances the 61.6–64.x driver-service preparation block with the operational contracts needed to manage driver dependencies, resource ownership and health without putting driver implementations back into CellKernel.

## 64.0 milestone — COMPLETE

The 64.0 gate freezes the last hardware-neutral access boundary before real device execution:

- [x] Overflow-safe MMIO region contract.
- [x] Overflow-safe PIO region contract.
- [x] Bounded sub-range containment for register/window access.
- [x] Explicit interrupt ownership contract with line/MSI/MSI-X modes.
- [x] Explicit power-state contract with deterministic bounded transitions.
- [x] Device-side access contract ties MMIO/PIO/interrupt ownership to one device identity.
- [x] Driver-side access plan ties hardware resources to one driver identity.
- [x] Driver service exports the access/power contracts.
- [x] Unit coverage for overflow, bounds, interrupt validity, power transitions and identity consistency.
- [x] No concrete PCI/ACPI/APIC/IOAPIC/VirtIO/DMA hardware execution counted early.

The detailed specification is `docs/MILESTONE_64_0.md`.

## Reserved for 65% checkpoint

The following are deliberately not counted toward 64% and remain the next major validation block:

- PCI/PCIe enumeration and BAR discovery;
- ACPI discovery and real power-resource execution;
- APIC/IOAPIC implementation and interrupt routing;
- VirtIO transport and queue execution;
- concrete VirtIO block/network/input/display drivers;
- real DMA/IOMMU hardware enforcement;
- QEMU hardware certification.

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

## 64% milestone boundary

The 64% milestone represents completion of the final hardware-neutral device-access preparation block from the master plan. CellKernel and driverd now have explicit, bounded contracts for MMIO/PIO access, interrupt ownership and power-state intent, all tied to a concrete device/driver identity. Real hardware discovery, interrupt routing, DMA programming and driver execution remain reserved for the 65% checkpoint.

## Definition of done

AWE_OS is considered product-ready only when the required boot, kernel, driver, security, recovery and release gates are implemented and their automated CI/QEMU validation is green. Documentation or architectural placeholders alone do not satisfy a gate.

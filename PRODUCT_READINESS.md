# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**50% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage is based on implemented product gates, not documentation or architectural stubs. It is recalculated after substantive repository changes.

## Hardware and driver intelligence

AWE_OS targets real hardware, virtual machines and cloud/server environments through one capability-controlled Driver HAL. It does not execute foreign kernel ABIs directly inside CellKernel.

Implemented foundations now include:
- cross-OS driver provenance manifests for Native AWE, Linux, Android, Windows, BSD and other ports;
- verified-only driver binding;
- bounded driver compatibility registry;
- offline/online driver database architecture with staged, authenticated and rollback-safe updates;
- bounded driver experience database that records probe outcomes and supports deterministic stability selection;
- fail-closed behavior for unknown/unverified hardware;
- hardened bootloader contract with validation, measurement and fail-closed goals.

## Implemented kernel foundations

- Intent-carrying authorization with required rights, impact class and resource budget.
- Deterministic security policy engine with explicit allow/deny reasons.
- Bounded, allocation-free causal provenance journal.
- Non-escalating capability derivation and explicit revocation state.
- Process states and consumable CPU, memory and IPC budgets.
- Allocation-free token-bucket rate limiter.
- Bounded scheduler queue and fixed-priority scheduling primitive.
- Typed physical/virtual addresses with overflow-safe alignment.
- Monotonic boot phases and terminal failure state.
- Hardware driver contracts for MMIO, DMA, interrupt mode and device identity.
- Bounded device-bus registry.
- VirtIO feature negotiation primitive requiring VirtIO 1.x before activation.
- AWE Capsule and XenoSense security foundations.
- Cloud CI security/release gates.

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
- [ ] ACPI tables and power-management discovery.
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

### AWE Bootloader security contract
- [x] AWE-only image contract documented.
- [x] Architecture and handoff validation requirements.
- [x] Memory-map normalization requirements.
- [x] Fail-closed malformed-image policy.
- [x] Measurement/signature extension point.
- [ ] Fully implemented x86_64 UEFI loader.
- [ ] Automated QEMU UEFI boot test.
- [ ] Signed release-image verification.
- [ ] TPM/Secure Boot integration.

## P0 — Boot
- [x] AWE boot magic/version and fixed-width ABI.
- [x] Loader validates architecture and handoff metadata.
- [x] Kernel exposes validated loader-to-kernel entry contract.
- [ ] Real x86_64 UEFI image boots in QEMU.
- [ ] Kernel reaches deterministic `Running` state in automated boot test.
- [x] Boot failure paths are diagnosable over serial console.

## P1 — Kernel
- [x] Rust `no_std` workspace.
- [x] Architecture abstraction.
- [x] Memory subsystem boundaries and typed addresses.
- [x] Scheduler/process/syscall/security boundaries.
- [x] Physical frame allocator.
- [x] x86_64 page-table structures.
- [ ] Page tables installed and activated on x86_64.
- [x] x86_64 IDT gate construction and `lidt` primitive.
- [x] x86_64 PIT programming primitive.
- [ ] IDT/timer active in boot path.
- [ ] Local APIC timer.
- [ ] SMP bring-up.
- [ ] Booted-kernel heap stress tests.

## P2 — Hardware/storage
- [x] Driver contracts and registry.
- [x] VirtIO feature negotiation.
- [ ] PCI/PCIe enumeration.
- [ ] ACPI discovery.
- [ ] VirtIO transport.
- [ ] VirtIO block/network.
- [ ] NVMe/AHCI.
- [ ] AWEFS/VFS with crash-safe metadata.
- [ ] USB/HID/framebuffer.

## P3 — User space
- [ ] Init/service manager.
- [ ] Process isolation and capability enforcement.
- [ ] Stable syscall ABI.
- [ ] IPC primitives.
- [ ] Native AWE Capsule runtime.
- [ ] Package verification and rollback-safe updates.
- [ ] Terminal and first-party utilities.

## P4 — Desktop
- [ ] Compositor/window server.
- [ ] AYUI foundation.
- [ ] Settings, file manager, terminal and system monitor.
- [ ] Network configuration UI.
- [ ] Installer and recovery environment.

## P5 — Production
- [ ] Reproducible release builds.
- [ ] Signed release images.
- [x] Cloud CI security/release gates.
- [x] Foundational unit/integration coverage.
- [ ] Real QEMU boot/release-image test.
- [ ] Fuzzing for boot/protocol parsers.
- [ ] Performance benchmarks and regression thresholds.
- [ ] Hardware compatibility matrix.
- [ ] Upgrade/recovery testing.
- [ ] Trusted computing base security review.

## Definition of done

A release is Product only when the reference image boots unattended in QEMU, initializes the kernel, starts user space, mounts persistent storage, brings up networking, launches a native application, and passes automated release gates.

Architecture documents and stubs are never counted as implemented features.

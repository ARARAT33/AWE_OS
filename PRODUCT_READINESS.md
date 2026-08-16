# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**48% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage is based on implemented product gates, not documentation or architectural stubs. It is recalculated after substantive repository changes.

## Hardware and driver direction

AWE_OS targets **real hardware, virtual machines and cloud/server environments**. VirtIO is the first virtualization target; PCI/ACPI and common PC hardware follow. Broad Linux hardware coverage is a strategic goal, implemented through a stable AWE Driver HAL and legally/technically reviewed ports or clean-room reimplementations rather than blindly embedding the Linux kernel.

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
- Hardware driver contract model for MMIO, DMA, interrupt mode and device identity.
- Bounded device-bus registry.
- VirtIO feature negotiation primitive requiring VirtIO 1.x before device activation.
- AWE Capsule and XenoSense specifications with kernel security foundations.
- Cloud CI security/release gates.

## Driver product roadmap

### Virtualization
- [x] Driver HAL and device contracts.
- [x] VirtIO feature negotiation foundation.
- [ ] VirtIO PCI transport.
- [ ] VirtIO block driver.
- [ ] VirtIO network driver.
- [ ] VirtIO console/entropy/input drivers.
- [ ] VirtIO GPU driver.
- [ ] Automated QEMU end-to-end device tests.

### PC hardware
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

### Linux-derived hardware coverage
- [ ] Driver-port compatibility layer.
- [ ] Linux driver provenance/license manifest.
- [ ] Automated device-ID coverage database.
- [ ] Priority ports for common Ethernet/Wi-Fi/storage/GPU/audio hardware.
- [ ] Hardware compatibility lab/matrix.

## Release gates

### P0 — Boot contract
- [x] AWE boot magic/version and fixed-width ABI.
- [x] Loader validates architecture and handoff metadata.
- [x] Kernel exposes validated loader-to-kernel entry contract.
- [ ] Real x86_64 UEFI image boots in QEMU.
- [ ] Kernel reaches deterministic `Running` state in automated boot test.
- [x] Boot failure paths are diagnosable over serial console.

### P1 — Kernel foundation
- [x] Rust `no_std` kernel workspace.
- [x] Architecture abstraction.
- [x] Memory subsystem boundaries and typed addresses.
- [x] Scheduler/process/syscall/security boundaries.
- [x] Physical frame allocator.
- [x] x86_64 page-table structures.
- [ ] Page tables installed and activated on x86_64.
- [x] x86_64 IDT gate construction and `lidt` primitive.
- [x] x86_64 PIT programming primitive.
- [ ] IDT/timer initialization active in boot path.
- [ ] Local APIC timer.
- [ ] SMP bring-up.
- [ ] Booted-kernel heap stress tests.

### P2 — Hardware and storage
- [x] Driver contracts and device registry foundation.
- [x] VirtIO feature negotiation foundation.
- [ ] PCI/PCIe enumeration.
- [ ] ACPI discovery.
- [ ] VirtIO transport.
- [ ] VirtIO block/network.
- [ ] NVMe/AHCI.
- [ ] AWEFS/VFS with crash-safe metadata.
- [ ] USB/HID/framebuffer.

### P3 — User space
- [ ] Init/service manager.
- [ ] Process isolation and capability enforcement.
- [ ] Stable syscall ABI.
- [ ] IPC primitives.
- [ ] Native AWE Capsule binary/package format.
- [ ] Package verification and rollback-safe updates.
- [ ] Terminal and first-party utilities.

### P4 — Desktop product
- [ ] Compositor/window server.
- [ ] AYUI foundation.
- [ ] Settings, file manager, terminal and system monitor.
- [ ] Network configuration UI.
- [ ] Installer and recovery environment.

### P5 — Production quality
- [ ] Reproducible release builds.
- [ ] Signed release images.
- [x] Cloud CI security/release gates.
- [x] Foundational kernel unit/integration coverage.
- [ ] Real QEMU boot/release-image test.
- [ ] Fuzzing for boot image/protocol parsers.
- [ ] Performance benchmarks and regression thresholds.
- [ ] Hardware compatibility matrix.
- [ ] Upgrade and recovery testing.
- [ ] Security review of the trusted computing base.

## Definition of done

A release is **Product** only when the reference image boots unattended in QEMU, initializes the kernel, starts user space, mounts persistent storage, brings up networking, launches a native application, and passes automated release gates.

Architecture documents and stubs are never counted as implemented features.

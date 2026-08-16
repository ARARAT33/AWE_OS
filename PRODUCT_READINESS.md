# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**40% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage is based on implemented product gates, not documentation or architectural stubs. It is recalculated after substantive repository changes.

## Security/performance principle

AWE_OS follows **small trusted core + bounded work + explicit capabilities + deterministic failure**. Experimental features must not silently modify the trusted computing base.

## New implemented kernel primitives

- Intent-carrying authorization with required rights, impact class and resource budget.
- Bounded, allocation-free causal provenance journal for security-sensitive events.
- Non-escalating capability derivation: child authority is the intersection of parent and requested rights.
- Explicit empty/revoked capability state.
- Process states and consumable CPU, memory and IPC budgets with underflow-safe rejection.
- AWE Capsule and XenoSense specifications now have corresponding kernel security foundations.
- Cloud security gate validates formatting, workspace compilation, tests, strict Clippy and the release contract on every main push/PR.

## Product target

**AWE_OS 1.0** is a Rust-first, capability-oriented operating system with a dedicated AWE boot chain, a stable loader-to-kernel ABI, a bootable x86_64 reference platform, isolated system services, a native application format, persistent storage, networking, graphics/input, and reproducible release images.

## Release gates

### P0 — Boot contract
- [x] AWE boot magic/version and fixed-width ABI.
- [x] Loader validates architecture and handoff metadata.
- [x] Kernel exposes a validated loader-to-kernel entry contract.
- [ ] Real x86_64 UEFI image boots in QEMU.
- [ ] Kernel reaches deterministic `Running` state in automated boot test.
- [x] Boot failure paths are diagnosable over serial console.

### P1 — Kernel foundation
- [x] Rust `no_std` kernel workspace.
- [x] Architecture abstraction.
- [x] Memory subsystem boundaries.
- [x] Scheduler/process/syscall/security module boundaries.
- [x] Physical frame allocator from loader memory map.
- [x] Four-level x86_64 page-table structures and address decomposition.
- [ ] Page tables installed and activated on x86_64.
- [x] x86_64 IDT gate construction and `lidt` primitive.
- [x] x86_64 PIT channel-0 programming.
- [ ] IDT/timer initialization active in boot path.
- [ ] Local APIC timer.
- [ ] SMP bring-up.
- [ ] Booted-kernel heap stress tests.
- [x] Intent authorization primitive.
- [x] Bounded provenance journal.
- [x] Non-escalating capability derivation/revocation model.
- [x] Consumable process resource budgets.

### P2 — Hardware and storage
- [ ] PCI/PCIe enumeration.
- [ ] ACPI discovery.
- [ ] VirtIO block/network.
- [ ] NVMe/AHCI.
- [ ] AWEFS/VFS with crash-safe metadata.
- [ ] Keyboard/mouse/framebuffer.

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

A release is **Product** only when the reference ISO/EFI image boots unattended in QEMU, initializes the kernel, starts user space, mounts persistent storage, brings up networking, launches a native application, and passes automated release gates.

Architecture documents and stubs are never counted as implemented features.

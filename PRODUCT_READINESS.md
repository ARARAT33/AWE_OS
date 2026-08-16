# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Current product readiness

**38% — AWE_OS 1.0 Product Readiness (current engineering estimate).**

The percentage is based on implemented product gates, not documentation or architectural stubs. It is recalculated after substantive repository changes.

## Security/performance principle

AWE_OS follows **small trusted core + bounded work + explicit capabilities + deterministic failure**. Experimental features must not silently modify the trusted computing base.

## New implemented kernel primitives

- Intent-carrying authorization: privileged requests can declare required rights, impact class and a non-zero resource budget before authorization succeeds.
- Bounded causal provenance journal: fixed-size, allocation-free security event storage suitable for the trusted path; overflow is bounded rather than allowed to allocate or block the kernel.
- Explicit process states and resource budgets for CPU ticks, memory bytes and IPC messages.
- AWE Capsule and XenoSense specifications now have corresponding kernel security foundations rather than remaining documentation-only concepts.

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
- [x] Process resource-budget model.

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
- [x] Automated QEMU smoke-test pipeline is defined.
- [ ] Fuzzing for boot image/protocol parsers.
- [x] Foundational kernel unit/integration coverage.
- [ ] Performance benchmarks and regression thresholds.
- [ ] Hardware compatibility matrix.
- [ ] Upgrade and recovery testing.
- [ ] Security review of the trusted computing base.

## Definition of done

A release is **Product** only when the reference ISO/EFI image boots unattended in QEMU, initializes the kernel, starts user space, mounts persistent storage, brings up networking, launches a native application, and passes automated release gates.

Architecture documents and stubs are never counted as implemented features.

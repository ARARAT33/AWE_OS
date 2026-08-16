# AWE_OS Product Readiness

This document defines the bar for calling AWE_OS a real bootable product rather than an architecture prototype.

## Product target

**AWE_OS 1.0** is a Rust-first, capability-oriented operating system with a dedicated AWE boot chain, a stable loader-to-kernel ABI, a bootable x86_64 reference platform, isolated system services, a native application format, persistent storage, networking, graphics/input, and reproducible release images.

## Release gates

### P0 — Boot contract

- [x] AWE boot magic/version and fixed-width ABI.
- [x] Loader validates architecture and handoff metadata.
- [x] Kernel exposes a validated loader-to-kernel entry contract.
- [ ] Real x86_64 UEFI image boots in QEMU.
- [ ] Kernel reaches a deterministic `Running` state.
- [ ] Boot failure paths are visible and diagnosable.

### P1 — Kernel foundation

- [x] Rust `no_std` kernel workspace.
- [x] Architecture abstraction exists.
- [x] Memory subsystem boundaries exist.
- [x] Scheduler/process/syscall/security module boundaries exist.
- [ ] Physical frame allocator is driven by the loader memory map.
- [ ] Virtual memory/paging is active on x86_64.
- [ ] IDT/APIC/timer initialization is active.
- [ ] SMP bring-up is active.
- [ ] Heap allocation is active and stress-tested.

### P2 — Hardware and storage

- [ ] PCI/PCIe enumeration.
- [ ] ACPI discovery.
- [ ] VirtIO block and network devices.
- [ ] NVMe/AHCI storage path for the reference hardware class.
- [ ] AWEFS/VFS with crash-safe metadata.
- [ ] Keyboard, mouse and framebuffer input/output.

### P3 — User space

- [ ] Init/service manager.
- [ ] Process isolation and capability enforcement.
- [ ] Stable syscall ABI.
- [ ] IPC primitives.
- [ ] Native AWE application manifest/package format.
- [ ] Package verification and rollback-safe updates.
- [ ] Terminal and first-party system utilities.

### P4 — Desktop product

- [ ] Compositor/window server.
- [ ] AYUI foundation.
- [ ] Settings, file manager, terminal and system monitor.
- [ ] Network configuration UI.
- [ ] Installer and recovery environment.

### P5 — Production quality

- [ ] Reproducible release builds.
- [ ] Signed release images.
- [ ] Automated QEMU smoke tests.
- [ ] Fuzzing for boot image/protocol parsers.
- [ ] Kernel unit/integration test suite.
- [ ] Performance benchmarks and regression thresholds.
- [ ] Hardware compatibility matrix.
- [ ] Upgrade and recovery testing.
- [ ] Security review of the trusted computing base.

## Definition of done

A release is **Product** only when the reference ISO/EFI image boots unattended in QEMU, initializes the kernel, starts user space, mounts persistent storage, brings up networking, launches a native application, and passes the automated release gates above.

Architecture documents and stubs are never counted as implemented features.

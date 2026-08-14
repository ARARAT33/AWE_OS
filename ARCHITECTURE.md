# AWEOS Architecture

AWEOS is designed as a Rust-first, modular, security-oriented operating system. The kernel is intentionally kept small and privileged; drivers, storage, networking, UI, compatibility layers, and applications are separate components.

## Principles

1. `no_std` kernel and direct hardware control.
2. Strong user/kernel isolation.
3. Capability-oriented permissions and least privilege.
4. Fast IPC using shared memory and carefully bounded channels where safe.
5. Drivers and system services isolated from the kernel where practical.
6. Native `.awosa` applications with signed manifests and explicit capabilities.
7. Compatibility layers are optional modules, never requirements for the native runtime.
8. Performance is measured with reproducible benchmarks rather than assumed.

## Layers

```text
Firmware / Boot
      |
AWE Bootloader
      |
CPU + SMP + Memory + Interrupts
      |
CellKernel
      |
IPC / Scheduler / Syscalls / Capabilities
      |
Drivers / Storage / Network / Services
      |
AWOSA Runtime + Compatibility Layers
      |
AYUI + Compositor + Terminal
      |
Native Applications
```

## Planned milestones

- x86_64 boot and early CPU initialization
- GDT/IDT and interrupt handling
- physical and virtual memory managers
- heap allocator and kernel synchronization primitives
- APIC/SMP and preemptive scheduler
- process/thread model and capability-based IPC
- syscall ABI
- PCI/PCIe, storage, input, display and network drivers
- AWEFS/VFS
- `.awosa` package/manifest/signature format
- user-space init/service manager
- terminal and AYUI compositor
- sandboxed native applications
- optional Linux/POSIX/Windows/Android compatibility layers
- reproducible build, test, boot and benchmark infrastructure

This document describes the target architecture; individual milestones are implemented incrementally and must be validated on real hardware and virtual machines.

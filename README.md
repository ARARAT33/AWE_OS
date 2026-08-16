# AWE_OS

AWE_OS is a Rust-first operating-system project focused on a small privileged kernel, a dedicated AWE boot chain, capability-oriented security, and modular user-space services.

## Current architecture

```text
Firmware / UEFI / BIOS
        |
    AWE Loader
        |
   CellKernel
        |
 CPU/SMP • Memory • Interrupts
        |
 Scheduler • Processes • Syscalls
        |
 IPC • Capabilities • Security
        |
 Drivers • Storage • Network
        |
 AWOSA runtime • AYUI • Services
        |
 Native AWE applications
```

The repository already contains an AWE-specific loader with architecture selection, ELF validation, identity checks, measurement, rollback protection, security policy, and UEFI support. The kernel is organized around architecture, memory, process, scheduler, security, synchronization, and syscall subsystems.

## Engineering rules

- Rust-first and `no_std` for privileged code.
- Keep hardware-dependent code behind architecture interfaces.
- Keep the kernel small; move drivers and services out of the trusted core where practical.
- Treat memory, process, IPC, and capability boundaries as security boundaries.
- Every boot or kernel feature must have a deterministic test or validation path where possible.
- Never describe a planned subsystem as implemented until it builds and is exercised.

## Roadmap to a real product

1. **Bootable x86_64 baseline** — reliable AWE loader → kernel handoff.
2. **CPU and memory foundation** — GDT/IDT, paging, physical-frame allocator, heap, SMP/APIC.
3. **Kernel execution model** — interrupts, scheduler, threads/processes, syscall ABI, IPC.
4. **Hardware platform** — PCI/PCIe, timers, storage, keyboard/input, framebuffer/display, networking.
5. **System services** — init/service manager, VFS/AWEFS, logging, time, device model.
6. **Native application platform** — AWOSA package/manifest/signature format, runtime, sandbox and capability enforcement.
7. **User experience** — terminal, compositor, AYUI and native applications.
8. **Compatibility** — optional Linux/POSIX/Windows/Android compatibility layers.
9. **Production validation** — QEMU boot tests, real-hardware matrix, reproducible builds, fuzzing, benchmarks and release artifacts.

## Status

This repository is an active operating-system foundation, not yet a general-purpose daily-driver OS. Architecture documents describe the target design; implementation status must be verified from source and CI.

## License

AGPL-3.0.

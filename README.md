# AWE_OS

AWE_OS is a Rust-first operating-system project focused on a small privileged kernel, a dedicated AWE boot chain, capability-oriented security, and modular user-space services.

## Product architecture

```text
UEFI / Firmware
      |
 AWE Loader
  identity • image validation • rollback policy
      |
 AWE Boot Protocol (versioned ABI)
      |
 CellKernel
  arch • memory • interrupts • scheduler • processes
  syscalls • IPC • capabilities • security
      |
 Device / System Services
  PCI • ACPI • storage • network • input • display
      |
 AWOSA Runtime
  packages • manifests • sandbox • capabilities
      |
 AYUI + Native Applications
```

## What is implemented today

- AWE-specific loader with architecture selection, ELF/image validation, identity checks, measurement, rollback controls, security policy and UEFI support.
- Versioned `BootInfo` protocol with explicit invariants.
- Kernel module boundaries for architecture, memory, interrupts, process, scheduler, security, synchronization and syscalls.
- Validated loader-to-kernel entry contract and architecture capability matrix.
- Automated formatting, workspace compilation, tests, Clippy and UEFI-loader checks in GitHub Actions.

## Engineering rules

- Rust-first and `no_std` for privileged code.
- Keep hardware-dependent code behind architecture interfaces.
- Keep the trusted computing base small; move drivers/services out of the kernel where practical.
- Treat firmware metadata, memory, process, IPC and capability boundaries as security boundaries.
- Every feature must have a deterministic test or validation path where possible.
- Never describe a planned subsystem as implemented until it builds and is exercised.

## Roadmap to AWE_OS 1.0

1. **Bootable x86_64** — UEFI/QEMU image and deterministic kernel entry.
2. **Memory + CPU** — physical frames, paging, heap, GDT/IDT, APIC/timers and SMP.
3. **Execution** — threads/processes, syscall ABI, IPC and capability enforcement.
4. **Hardware** — PCI/PCIe, ACPI, VirtIO, storage, input, framebuffer and networking.
5. **System services** — init/service manager, VFS/AWEFS, device model, time and logging.
6. **Native platform** — AWOSA package/manifest/signature format, runtime and sandbox.
7. **Desktop** — terminal, compositor, AYUI, file manager, settings and system monitor.
8. **Compatibility** — optional Linux/POSIX/Windows/Android compatibility layers.
9. **Production** — signed images, reproducible builds, fuzzing, benchmarks, recovery and hardware matrix.

See **[PRODUCT_READINESS.md](PRODUCT_READINESS.md)** for the release gates and **[docs/ABI.md](docs/ABI.md)** for the boot ABI contract.

## Status

AWE_OS is **not yet a finished daily-driver OS**. The foundation is materially stronger, but the product gate requires a real bootable x86_64 image, active hardware drivers, persistent storage, networking, user space and automated end-to-end QEMU validation.

## License

AGPL-3.0.

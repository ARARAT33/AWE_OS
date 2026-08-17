# AWE_OS

AWE_OS is a Rust-first operating-system project focused on a small privileged CellKernel, a dedicated AWE boot chain, capability-oriented security, and modular user-space services.

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
 AWE Capsule
  executable • permissions • provenance • budgets • recovery
      |
 AYUI + Native Applications

Experimental research layer:
  AWE XenoSense
  intent • causal provenance • semantic recovery
  uncertainty-aware scheduling • hardware contracts
```

## What is implemented today

- AWE-specific loader with architecture selection, ELF/image validation, identity checks, measurement, rollback controls, security policy and UEFI support.
- Versioned `BootInfo` protocol with explicit invariants.
- Kernel module boundaries for architecture, memory, interrupts, process, scheduler, security, synchronization and syscalls.
- Validated loader-to-kernel entry contract and architecture capability matrix.
- A typed CellKernel system contract with an explicit ABI version and deterministic capability baseline.
- Automated formatting, workspace compilation, tests, Clippy and UEFI-loader checks in GitHub Actions.
- Experimental product specifications for AWE Capsule and AWE XenoSense.

## Engineering documentation

- **[ENGINEERING_GUIDE.md](docs/ENGINEERING_GUIDE.md)** — source-of-truth map, invariants, development loop and release evidence requirements.
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — system layers and long-term architecture.
- **[PRODUCT_READINESS.md](PRODUCT_READINESS.md)** — evidence-based 1.0 gates and current readiness.
- **[AWE_OS_100_PERCENT_MASTER_PLAN.md](docs/AWE_OS_100_PERCENT_MASTER_PLAN.md)** — complete 1.0 implementation plan.
- **[ABI.md](docs/ABI.md)** — boot ABI contract.
- **[BOOT_ARCHITECTURE.md](docs/BOOT_ARCHITECTURE.md)** — loader and boot flow.
- **[DRIVER_SYSTEM.md](docs/DRIVER_SYSTEM.md)** — driver architecture.
- **[AWE_CAPSULE.md](docs/AWE_CAPSULE.md)** — native application security model.
- **[AWE_XENOSENSE.md](docs/AWE_XENOSENSE.md)** — experimental research concepts.

## New research direction — AWE XenoSense

AWE XenoSense is an experimental OS research layer exploring ideas such as Intent-Carrying Execution, Causal Provenance Graphs, Time-Travel Kernel State, Uncertainty-Aware Scheduling, Memory Intent Zones, Self-Describing Hardware Contracts and Proof-Carrying Recovery.

These are **design proposals, not claims of proven technology or claims that nobody else has independently explored similar ideas**. They are deliberately designed to remain subordinate to AWE_OS's deterministic and capability-based security model.

## Engineering rules

- Rust-first and `no_std` for privileged code.
- Keep hardware-dependent code behind architecture interfaces.
- Keep the trusted computing base small; move drivers/services out of the kernel where practical.
- Treat firmware metadata, memory, process, IPC and capability boundaries as security boundaries.
- Every feature must have a deterministic test or validation path where possible.
- Never describe a planned subsystem as implemented until it builds and is exercised.
- Experimental intelligence/recovery features must never silently rewrite the trusted computing base.

## Roadmap to AWE_OS 1.0

1. **Bootable x86_64** — UEFI/QEMU image and deterministic kernel entry.
2. **Memory + CPU** — physical frames, paging, heap, GDT/IDT, APIC/timers and SMP.
3. **Execution** — threads/processes, syscall ABI, IPC and capability enforcement.
4. **Hardware** — PCI/PCIe, ACPI, VirtIO, storage, input, framebuffer and networking.
5. **System services** — init/service manager, VFS/AWEFS, device model, time and logging.
6. **Native platform** — AWOSA package/manifest/signature format, runtime and sandbox.
7. **AWE Capsule** — native application identity, capability, resource and recovery contract.
8. **Desktop** — terminal, compositor, AYUI, file manager, settings and system monitor.
9. **Compatibility** — optional Linux/POSIX/Windows/Android compatibility layers.
10. **Production** — signed images, reproducible builds, fuzzing, benchmarks, recovery and hardware matrix.

## Status

AWE_OS is **not yet a finished daily-driver OS**. The foundation is materially stronger, but the product gate requires a real bootable x86_64 image, active hardware drivers, persistent storage, networking, user space and automated end-to-end QEMU validation.

## License

AGPL-3.0.

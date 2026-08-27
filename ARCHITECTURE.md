# AWE_OS CellKernel Architecture

**Architecture status:** Active development baseline  
**Project track:** AWE development project #2

AWE_OS is designed as a Rust-first, modular, security-oriented operating system. CellKernel is intentionally small and privileged; drivers, storage, networking, UI, compatibility layers, and applications remain separate components where practical.

## Architectural goal

The architecture is a living implementation target. The priority is to move each boundary from specification to executable behavior while preserving stable contracts and keeping the trusted computing base small.

## Principles

1. `no_std` kernel and direct hardware control where required.
2. Strong user/kernel isolation.
3. Capability-oriented permissions and least privilege.
4. Fast, bounded IPC with explicit ownership rules.
5. Drivers and system services isolated from the kernel where practical.
6. Native applications use explicit manifests and capabilities.
7. Compatibility layers are optional modules, never prerequisites for the native runtime.
8. Performance and reliability are measured with reproducible tests and runtime evidence.
9. ABI and security contracts are versioned and validated at boundaries.
10. Experimental features must not silently modify the trusted computing base.

## Layers

```text
Firmware / Boot
      |
AWE Loader
      |
AWE Boot Protocol
      |
CPU + SMP + Memory + Interrupts
      |
CellKernel
      |
Syscalls / IPC / Scheduler / Capabilities
      |
Drivers / Storage / Network / System Services
      |
AWOSA Runtime
      |
AYUI / Compositor / Terminal
      |
Native Applications
```

## Current development priorities

### 1. Executable core

- x86_64 UEFI/QEMU boot
- deterministic kernel entry
- physical/virtual memory and paging
- GDT/IDT, timers and interrupt execution
- process/thread model and scheduler execution
- syscall and capability enforcement

### 2. Hardware and services

- PCI/PCIe and ACPI runtime
- APIC/IOAPIC and VirtIO execution
- storage and filesystem runtime
- input and display paths
- network runtime
- userspace init/service management

### 3. Native platform

- AWOSA runtime integration
- `.asd` driver package lifecycle
- `.awos` application package lifecycle
- cryptographic signing and verification
- sandbox and resource policy enforcement
- A/B update and recovery execution

### 4. Desktop and ecosystem

- AYUI compositor/windowing
- terminal and first-party applications
- file and settings applications
- SDK/App Builder
- optional compatibility layers

## Completion model

A subsystem has three distinct states:

- **Designed** — architecture and contracts exist.
- **Implemented** — code exists and local tests/build checks pass.
- **Validated** — the subsystem has executable runtime evidence at the relevant integration boundary.

Only the third state should be used for product-release claims.

## Documentation hierarchy

- `PROJECT_STATUS.md` — current project state and development direction.
- `PRODUCT_READINESS.md` — release/readiness gates.
- `docs/AWE_OS_100_PERCENT_MASTER_PLAN.md` — long-term implementation plan.
- Other `docs/*PROGRESS*`, `*EVIDENCE*`, and dated documents — historical or milestone-specific records.

This hierarchy prevents older progress documents from being mistaken for the current state of `main`.

## Long-term target

AWE_OS should evolve into a reproducible, secure, modular operating system whose core can be independently validated on virtual and real hardware. The architecture deliberately favors incremental, testable progress over unsupported completion percentages.

# AWEOS Ecosystem

AWEOS uses a small set of stable contracts so components can evolve independently without turning the kernel into a monolith.

## Core contracts

```text
BootInfo -> Kernel
Kernel ABI -> Services
Capability ABI -> IPC
Device ABI -> Drivers
VFS ABI -> Filesystems
Process ABI -> Runtimes
Display/Input ABI -> AYUI
Package ABI -> AWOSA
```

Every contract has a version, feature bits, explicit ownership and failure semantics.

## Component graph

```text
                     AWEOS SDK
                         |
              +----------+----------+
              |          |          |
           AWESA        AWOSA      Tools
              |          |          |
              +---- AWE Service Bus-+
                         |
        +----------------+----------------+
        |                |                |
    CellKernel        Drivers          Services
        |                |                |
        +----------------+----------------+
                         |
                    Hardware HAL
```

The service bus is not a mandatory central broker for every operation. High-frequency paths use direct capability channels or shared-memory queues; the bus is used for discovery, lifecycle and low-frequency coordination.

## SDK

The future AWEOS SDK should expose architecture-neutral interfaces for:

- processes and threads
- capabilities
- IPC
- filesystem
- networking
- graphics/input
- timers
- devices
- cryptography
- package management

Native applications should not need to know whether the host CPU is x86_64, AArch64 or RISC-V.

## Driver portability

A driver is split into:

1. portable protocol logic
2. architecture-independent device logic
3. architecture/SoC-specific backend
4. firmware/device-description adapter

This allows the same protocol implementation to serve multiple controller families.

## Compatibility environments

Compatibility runtimes are first-class services but are isolated from the native ABI:

- Linux/POSIX runtime
- Windows API/runtime
- Android runtime

They may translate calls to AWEOS services or execute a contained guest environment when API translation is insufficient. This is the path to broad application compatibility without contaminating the native kernel ABI.

## Packaging

`.awosa` is the native package format. It contains a manifest, executable, resources, declared capabilities and optional signature. Reproducible builds and content-addressed package identities are planned.

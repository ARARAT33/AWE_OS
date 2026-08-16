# AWE_OS Driver Strategy

AWE_OS is being designed to run on three classes of machines:

1. **Real hardware** — x86_64 PCs and later ARM64/RISC-V targets.
2. **Virtual machines** — QEMU/KVM, VMware, VirtualBox and other environments where standard virtual hardware can be exposed.
3. **Cloud/servers** — VirtIO-first environments, followed by platform-specific devices.

## Linux driver compatibility strategy

The goal is broad Linux hardware coverage, but AWE_OS will **not blindly copy the Linux kernel or every Linux driver into the kernel**. Linux drivers have their own kernel APIs, dependencies, configuration model and licensing obligations.

Instead, AWE_OS uses a layered porting strategy:

```text
Linux hardware knowledge / specifications
              |
      AWE Driver HAL
              |
   +----------+----------+
   |                     |
Native AWE drivers   Linux-driver ports
   |                     |
   +----------+----------+
              |
        Device Manager
              |
       Capability boundary
              |
          CellKernel
```

Drivers should be ported or reimplemented against the AWE Driver HAL. Where a Linux driver can legally and technically be reused, its license and provenance must be preserved and its Linux dependencies replaced with explicit AWE interfaces.

## Driver coverage roadmap

### Tier 0 — mandatory virtual hardware

- VirtIO transport and common feature negotiation
- VirtIO block
- VirtIO network
- VirtIO console
- VirtIO entropy
- VirtIO input
- VirtIO GPU
- PCI transport
- ACPI discovery
- generic timers and interrupt controllers

### Tier 1 — common PC hardware

- x86_64 APIC/IOAPIC
- AHCI
- NVMe
- USB xHCI
- HID keyboard/mouse
- PS/2 fallback
- Intel/AMD graphics paths where feasible
- common Ethernet controllers
- common Wi-Fi chipsets
- audio controllers

### Tier 2 — broad Linux-derived hardware coverage

Prioritize hardware by real-world prevalence and maintainability. Each port must define:

- PCI/device identifiers
- MMIO/PIO regions
- DMA constraints
- interrupt mode
- power-management behavior
- reset/recovery path
- security capabilities
- resource limits
- test hardware or emulator coverage

## Security requirements for every driver

A driver must not receive ambient access to arbitrary MMIO, DMA or interrupts. Its device contract must explicitly declare the resources it needs. The device manager validates the contract before activation.

DMA must be bounded by an IOMMU when available. MMIO access must stay inside declared regions. Driver failure must be isolated from the rest of the system wherever architecture permits.

## Performance requirements

- zero-copy I/O where safe
- bounded queues
- interrupt moderation where appropriate
- batched DMA submissions
- per-device resource budgets
- lock minimization in hot paths
- no allocation in interrupt-critical paths

## Virtualization-first validation

QEMU is the first automated hardware target. The driver test matrix will progressively cover:

`VirtIO → PCI → ACPI → NVMe/AHCI → xHCI → networking → GPU/input → SMP`

A driver is not marked production-ready merely because it compiles; it must pass an emulator or hardware-backed end-to-end test.

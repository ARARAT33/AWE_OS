# AWEOS Universal Platform

AWEOS targets one architecture with portable hardware backends rather than pretending that one binary can natively contain every device driver.

## CPU families

- x86_64
- x86 (32-bit)
- AArch64
- ARM (32-bit)
- RISC-V 64
- RISC-V 32

The architecture layer owns CPU startup, exception/interrupt entry, atomics, MMU/page-table operations, timers, context switching and low-level synchronization. Higher layers use architecture-neutral interfaces.

## Real hardware strategy

Hardware support is divided into stable interfaces and independently testable drivers:

- PCI/PCIe discovery
- ACPI/UEFI on PC-class systems
- device-tree on ARM/RISC-V platforms
- USB host/device
- NVMe and AHCI/SATA
- eMMC/SD
- Ethernet/Wi-Fi
- HID keyboard/mouse/touch
- framebuffer/display and future GPU backends
- audio
- camera/sensors where documented interfaces exist
- power management

Drivers must not be trusted merely because they are loaded. Where hardware permits, drivers run in isolated processes/services and receive explicit capabilities.

## Compatibility

AWEOS keeps native AWOSA applications separate from compatibility environments:

```text
AWOSA -> native AWEOS ABI
Linux -> Linux ABI/runtime layer
Windows -> Windows API/runtime layer
Android -> Android runtime/ABI layer
```

A compatibility layer may reuse a compatible open-source implementation when its license and security model permit it. Proprietary Windows, Android or vendor drivers cannot simply be copied into AWEOS; hardware-specific drivers must be independently implemented or legally redistributable.

## Device classes

The target is broad real-device coverage, but support is introduced by device class and verified on physical hardware. Every driver should have a capability declaration, reset/recovery path, timeout policy and test harness.

## Boot

The portable boot interface receives a normalized `BootInfo` structure containing memory map, CPU topology, firmware tables, framebuffer and device-tree/ACPI references. Architecture-specific boot adapters translate UEFI, BIOS-compatible loaders and device-tree firmware into that common structure.

The bootloader is intentionally kept small. It loads a signed kernel image and optional modules, validates their metadata, establishes the initial CPU state and transfers control to the architecture-specific kernel entry point.

## Replacement goal

AWEOS aims to be a general-purpose replacement platform for desktop, server, embedded and mobile workloads. This is a long-term engineering goal, not a claim that every device or application is already supported. Compatibility is measured with hardware matrices, ABI tests, application tests and reproducible benchmarks.

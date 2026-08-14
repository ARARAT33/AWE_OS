# AWEOS Universal Platform

AWEOS targets a single kernel architecture with per-architecture boot and hardware backends.

## CPU targets

- x86_64
- AArch64
- RISC-V 64
- RISC-V 32
- future 32-bit x86/ARM backends where hardware support is practical

The kernel API is architecture-neutral. Privileged instructions, page tables, interrupt controllers, timers and context switching remain in architecture backends.

## Device model

AWEOS uses a capability-based driver model. A driver receives only the device resources it needs. Drivers should run outside the most privileged kernel code whenever the platform allows it.

The compatibility goal is to support existing Linux, Windows and Android hardware through dedicated driver/ABI adapters rather than copying entire foreign kernels into AWEOS. Native AWEOS drivers remain the preferred path for maximum performance and security.

## Application compatibility

Native `.awosa` applications use the AWEOS ABI directly. Foreign applications are supported by optional compatibility environments:

- Linux ELF + syscall/ABI environment
- Windows PE/Win32 compatibility environment
- Android application/runtime environment

Compatibility layers are isolated so that they cannot silently gain kernel privileges.

## Devices

The long-term device strategy is:

1. native AWEOS drivers for common hardware;
2. firmware/standard-bus discovery through ACPI, UEFI, PCI/PCIe, USB, NVMe and standard device protocols;
3. isolated compatibility drivers where licensing and technical constraints permit;
4. userspace driver services whenever practical.

No OS can honestly guarantee every Linux/Windows/Android driver or every application without implementing the corresponding hardware protocols, ABIs, firmware expectations and legal redistribution requirements. AWEOS therefore treats compatibility as an independently testable subsystem.

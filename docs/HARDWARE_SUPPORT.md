# Hardware Support Matrix

AWEOS uses explicit capability-based drivers and architecture backends. This matrix is a living target, not a claim of completed support.

| Area | Target |
|---|---|
| x86 | 32/64-bit |
| ARM | 32/64-bit |
| RISC-V | 32/64-bit |
| Firmware | UEFI, BIOS-compatible boot paths, device tree |
| Storage | NVMe, AHCI/SATA, eMMC, SD |
| Buses | PCI/PCIe, USB |
| Input | HID keyboard, mouse, touch |
| Network | Ethernet, Wi-Fi through supported controller drivers |
| Display | framebuffer first, accelerated GPU backends incrementally |
| Audio | standard controller families incrementally |
| Mobile | ARM SoCs through device-tree and vendor-neutral drivers where documented |

## Rule for real hardware

No hardware feature is considered complete until it boots and passes a physical-device test. Virtual-machine support is useful for development but is not equivalent to hardware validation.

## Driver contract

Each driver should expose:

1. discovery/probe
2. initialization
3. interrupt/event handling
4. bounded I/O
5. reset/recovery
6. shutdown
7. capability declaration
8. observability and diagnostics

The driver API must remain architecture-neutral whenever possible.

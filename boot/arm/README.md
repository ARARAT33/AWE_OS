# ARM Boot

ARM boot is firmware/platform specific. AWEOS uses the same BootInfo contract after entry, while adapters consume device-tree/firmware-provided memory and CPU information.

Targets:

- ARMv7 / 32-bit
- AArch64 / 64-bit

The loader must not assume PC-style ACPI on ARM. Device Tree and platform firmware are first-class boot inputs.

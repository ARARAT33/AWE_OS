# x86 Boot

This directory contains the x86 firmware entry path. BIOS stage-1 is deliberately tiny and transfers to a stage-2 loader. UEFI uses a separate PE/COFF EFI application and does not share the BIOS execution mode.

Targets:

- x86 32-bit
- x86_64
- BIOS-compatible boot path
- UEFI boot path (implementation follows after the common protocol)

The architecture-neutral handoff is defined in `boot/protocol`.

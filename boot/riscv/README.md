# RISC-V Boot

AWEOS supports both RV32 and RV64 as architectural targets. Platform firmware and boot ROM conventions vary, so the loader uses a small platform adapter and hands a normalized BootInfo to CellKernel.

Targets:

- RISC-V 32-bit
- RISC-V 64-bit

The kernel owns the final SATP/MMU setup after the loader has supplied memory and firmware metadata.

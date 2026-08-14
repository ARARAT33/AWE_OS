# AWEOS Bootloader

The boot system is split into small firmware/architecture adapters and one stable kernel handoff protocol.

## Supported architecture targets

- x86 32-bit
- x86_64
- ARM 32-bit
- AArch64
- RISC-V 32-bit
- RISC-V 64-bit

## Security requirements

The production loader will verify the kernel image and every privileged module before execution. It will reject malformed metadata, overlapping memory claims and unsupported ABI versions. Recovery and diagnostic paths must not silently execute untrusted privileged code.

## Current implementation status

The repository now contains:

- a versioned `BootInfo` ABI
- architecture-neutral loader handoff
- low-level Rust primitives for six CPU targets
- x86 BIOS stage-1 entry
- platform-specific boot design documents

A complete production bootloader additionally requires firmware-specific implementations, filesystem/image loading, cryptographic verification, memory-map acquisition and physical-hardware validation for each target family. Those pieces are intentionally implemented incrementally rather than represented as fake universal support.

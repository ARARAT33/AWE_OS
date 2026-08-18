# PCI → VirtIO probe bridge

## Scope

This milestone adds the bounded integration boundary between the existing PCI config-space enumerator and the modern VirtIO PCI transport validator.

The bridge performs deterministic admission and translation only. It does **not** perform unsafe MMIO access or claim hardware runtime evidence.

## Implemented

- Reject non-VirtIO PCI functions before transport construction.
- Classify the currently supported VirtIO network (`0x1041`) and block (`0x1042`) device IDs.
- Convert enumerated BAR values into validated transport windows.
- Reject zero-sized or invalid BAR windows.
- Preserve the existing modern-transport and queue-capacity checks.
- Exercise feature negotiation, queue configuration and driver-ready transitions through the validated transport contract.
- Keep probe output bounded and deterministic.

## Evidence boundary

Unit tests prove the PCI-to-transport contract using an in-memory `PciFunction`. They do **not** prove real PCI config-space access, physical BAR discovery, device DMA, or QEMU device execution.

Those remain release-gate work and must be demonstrated by runtime/QEMU CI before the corresponding Master Plan items can be marked complete.

## Next runtime step

1. Connect the platform PCI config-space implementation to `PciEnumerator`.
2. Discover the real VirtIO PCI capability list and BAR sizes.
3. Bind the probe result to the hardware register-access layer.
4. Exercise a VirtIO block/network queue under QEMU.
5. Record logs/artifacts in release evidence and make the gate CI-enforced.

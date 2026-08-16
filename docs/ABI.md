# AWE_OS ABI Contract

The loader-to-kernel boundary is intentionally small and versioned.

## BootInfo invariants

1. `magic` must equal `AWE_BOOT_MAGIC`.
2. `version` must equal `AWE_BOOT_VERSION`.
3. `size` must be at least the compiled `BootInfo` size.
4. CPU count must be non-zero.
5. Architecture must be one of the currently supported 64-bit architectures.
6. A non-zero memory-region count requires a non-null memory-region pointer.
7. The kernel must validate the structure before dereferencing any firmware-owned pointer.

## Compatibility policy

- New fields are appended to `BootInfo`.
- Existing field offsets and meanings are never repurposed within a major ABI version.
- A loader must reject an ABI version it cannot safely interpret.
- A kernel must ignore optional fields that are outside the supplied `size`.
- Release images must record the exact boot protocol version.

## Security policy

The boot chain treats all firmware-provided pointers and metadata as untrusted input until validated. Image identity, architecture, range checks, rollback policy and signature verification are separate gates so each can be tested independently.

## Product requirement

Before AWE_OS 1.0, the ABI must be exercised by an end-to-end UEFI/QEMU boot test rather than only unit tests.

# AWE_OS A→J implementation progress — VirtIO PCI transport boundary

Date: 2026-08-18

This milestone adds a real, bounded Rust implementation for the next D-stage hardware boundary. It does **not** claim hardware certification.

## Implemented in this commit

- PCI identity matching with explicit VirtIO vendor detection.
- BAR validation: non-zero power-of-two size, alignment, overflow-safe end calculation, and I/O-address bounds.
- Modern VirtIO capability validation with mandatory common/notify windows.
- Bounded queue-count admission.
- VirtIO feature negotiation wired to the existing transport state machine.
- Driver-ready transition after successful feature and queue configuration.
- Deterministic unit tests for rejection, BAR safety, negotiation, and queue bounds.

## What remains intentionally open

This is a **transport contract and validation boundary**, not yet a hardware driver. The master plan still requires real PCI config-space enumeration, BAR discovery from hardware, VirtIO PCI register access, device-specific block/network/console drivers, QEMU device exercise, and hardware evidence before D can be certified.

The repository master plan defines 100% as release certification and requires implementation, tests, runtime/emulator evidence, CI, recovery/error handling, and documentation. See `docs/AWE_OS_100_PERCENT_MASTER_PLAN.md` and `docs/PRODUCT_GAP_MATRIX.md`.

## Evidence policy

No checklist item is marked complete by this document. CI status must be observed from GitHub Actions after the commit is published. Hardware/runtime certification remains pending until the corresponding gates are green.

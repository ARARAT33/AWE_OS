# A→J VirtIO hardening — 2026-08-18

## Implemented

This increment strengthens the existing VirtIO/device-platform contracts without claiming hardware certification.

### Driver transport

- Added fail-closed PCI identity validation for VirtIO vendor `0x1AF4` and the modern device-id range `0x1000..=0x107F`.
- Added a bounded queue pending-distance check so corrupted avail/used indices cannot silently become an accepted queue state.
- Added deterministic unit tests for invalid vendor/device identities and queue corruption.

### VirtIO block

- Added explicit VirtIO block completion status constants (`OK`, `IOERR`, `UNSUPP`).
- Completion status is now validated before accepting a completion.
- Added mapping from device completion status to typed driver errors.
- Preserved bounded request size, capacity and descriptor validation.
- Added tests for I/O error, unsupported-operation and malformed completion paths.

### CI/evidence

- Added `cargo fmt --all -- --check` to the VirtIO runtime gate.
- Added a reproducible artifact manifest containing commit, runner, QEMU, Rust and artifact hashes.
- QEMU runtime evidence still requires QMP negotiation, PCI enumeration and a VirtIO PCI device before the gate can pass.

## Evidence boundary

This increment is **implementation/test hardening**, not release certification. It does not claim that AWE_OS has completed real persistent block I/O, DMA/IOMMU enforcement, NVMe/AHCI, physical-hardware validation, recovery testing, or the full A→J product plan.

The master plan defines 100% as a release certification state requiring implementation, tests, runtime/emulator evidence, CI, recovery/error handling and documentation. See `docs/AWE_OS_100_PERCENT_MASTER_PLAN.md`.

## Next high-value gate

`PCI identity -> transport initialization -> queue setup -> real VirtIO block request -> device completion -> persistent read/write -> recovery -> QEMU/CI evidence`

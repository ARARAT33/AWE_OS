# A/J VirtIO Block Completion Evidence

Date: 2026-08-18

## Implemented

The VirtIO block queue now preserves the device completion status from submission through `poll_completion()` instead of synthesizing every completion as `VIRTIO_BLK_S_OK`.

The queue also:

- rejects descriptors shorter than the VirtIO block header budget;
- bounds descriptor length against the maximum block request payload plus header;
- rejects an already-pending request slot;
- validates completion status against `OK`, `IOERR`, and `UNSUPP`;
- rejects completion bytes above the configured maximum request size;
- propagates `IOERR` and `UNSUPP` as typed block errors.

## Test evidence

The unit suite contains explicit coverage for:

1. bounded block requests;
2. arithmetic overflow and capacity escape;
3. invalid descriptor and completion rejection;
4. preservation of `IOERR` through queue completion;
5. successful queue submission/notification and completion.

## Evidence boundary

This change is implementation/test evidence only. It does **not** claim persistent guest-visible disk I/O, physical hardware execution, DMA/IOMMU enforcement, signed-driver trust, or release certification.

Per `AWE_OS_100_PERCENT_MASTER_PLAN.md`, progress is certified only after implementation, tests, runtime/emulator evidence, CI, recovery/error handling, and documentation are all satisfied.

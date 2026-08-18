# AWE_OS VirtIO QEMU evidence gate

## Purpose

This document defines the evidence produced by the VirtIO runtime CI gate. It is intentionally narrower than a release certification claim.

## Required checks

1. Workspace storage/driver contract tests pass.
2. The x86_64 kernel builds from the pinned CI toolchain.
3. The kernel artifact passes Multiboot2 validation.
4. QEMU exposes `virtio-blk-pci`.
5. QEMU is started with a real VirtIO block PCI device backed by the CI disk image.
6. QMP `query-pci` is captured while the guest is running and must contain VirtIO device evidence.
7. AWE_OS boot/runtime markers are present in the guest serial log.
8. SHA-256 digests are published for the ISO and test disk image.
9. QEMU serial/debug logs and QMP evidence are uploaded as CI artifacts.

## Evidence boundary

Passing this gate proves that CI can construct a QEMU environment containing a VirtIO block PCI device and that the AWE_OS guest reaches its required boot markers while that device is present. It does not by itself prove successful AWE_OS VirtIO block I/O, DMA/IOMMU isolation, persistent filesystem semantics, or hardware-in-the-loop behavior.

Those capabilities remain separate Master Plan release gates and must not be marked complete without their own runtime evidence.

## Failure policy

The gate is fail-closed: missing QMP evidence, missing guest markers, a failed contract test, invalid kernel artifact, or missing VirtIO device support fails the workflow.

## Reproducibility artifacts

Every run publishes:

- `artifact-sha256.txt`
- `qemu-device-evidence.json`
- `qemu-virtio.log`
- `qemu-host.log`
- `qemu-virtio-debug.log`
- `aweos-x86_64.iso`
- `virtio-disk.img`

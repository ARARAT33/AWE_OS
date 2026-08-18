# AWE_OS Release Evidence Ledger

This document records release evidence without converting planned work into fake completion.

## Certification rule

A feature is release-certified only when implementation, tests, runtime/emulator evidence, CI, recovery/error handling, and documentation are all present. This follows `docs/AWE_OS_100_PERCENT_MASTER_PLAN.md`.

## Current evidence snapshot

| Gate | Current evidence | Certification |
|---|---|---|
| Rust workspace formatting/check/tests | CI workflow configured; current revision requires a new green run | Pending latest green run |
| Clippy | CI workflow configured; current revision requires a new green run | Pending latest green run |
| UEFI loader check | CI workflow configured | Pending latest green run |
| x86_64 boot image | `boot-image.yml` builds ISO and validates ELF | Pending latest green run |
| QEMU boot smoke | Workflow checks boot/running/alive markers | Pending latest green run |
| Kernel dependency isolation | `kernel/Cargo.toml` contains only boot protocol dependency | Implemented by current source structure |
| A-C bounded runtime | `ac_boot_gate`, `ac_runtime` and `ac_completion` with deterministic tests | Implementation evidence present; runtime certification pending |
| A-C bring-up ordering | `BringupGate` covers BootInfo -> GDT/TSS -> IDT -> interrupts -> APIC -> memory -> paging -> heap -> SMP -> kernel-ready | Implementation + unit-test evidence present; hardware execution pending |
| A-C execution boundaries | Context-frame, syscall, IPC quota, capability generation, timer and trace validation are bounded and fail-closed | Implementation + unit-test evidence present; runtime certification pending |
| Driver dependency cycle rejection | `DependencyGraph` rejects self and transitive cycles | Implementation + unit-test evidence present |
| Driver lifecycle/recovery | `DriverSupervisor` plus `DriverLifecycle` enforce deterministic transitions, bounded restart budget and explicit quarantine | Implementation + unit-test evidence present; runtime certification pending |
| PCI enumeration boundary | `PciEnumerator` now has bounded multi-bus scanning plus validated PCI mechanism-#1 BDF/config-address construction; platform I/O remains an explicit adapter boundary | Implementation + unit-test evidence present; hardware enumeration pending |
| PCI BAR resource validation | `decode_bar` validates I/O and 32/64-bit memory BAR size, alignment, overflow and 64-bit upper-pair requirements | New implementation + unit-test evidence present; hardware BAR probing pending |
| VirtIO PCI transport boundary | Identity, BAR/capability/queue/feature validation and driver-ready state | Implementation + unit-test evidence present; hardware register exercise pending |
| PCI → VirtIO bridge | `VirtioPciProbe` classifies supported VirtIO functions and translates validated BAR windows into transport state without unsafe MMIO writes | Implementation + unit-test evidence present; physical device exercise pending |
| VirtIO block request plane | Bounded sector validation, DMA-bounded descriptor submission, queue completion and interrupt acknowledgment contract | Implementation + unit-test evidence present; persistent VirtIO device exercise pending |
| VirtIO block QEMU gate | `virtio-runtime.yml` runs workspace tests, builds the x86_64 image, boots QEMU with a `virtio-blk-pci` device, and uploads serial/debug evidence | Pending green CI run; device exercise is not yet equivalent to successful guest I/O |
| Storage GPT path | Bounded GPT scan is exercised through `BlockDevice` | Implementation + unit-test evidence present; persistent-device certification pending |
| Storage crash recovery | `JournalTxn` models prepare/commit/abort and converts non-durable states to `NeedsRecovery` | Implementation + unit-test evidence present; crash-injection evidence pending |
| Networking packet core | Ethernet/ARP/IPv4 routing and bounded UDP/TCP metadata validation | Implementation + unit-test evidence present; runtime certification pending |
| Network security policy | Allocation-free deny-by-default firewall with bounded rule capacity | Implementation + unit-test evidence present; runtime certification pending |
| Cryptographic signing | Required by master plan | Not certified |
| Hardware-in-loop matrix | Required by master plan | Not certified |
| Fuzz/stress/resource exhaustion | Required by master plan | Not certified |
| Signed reproducible release artifacts | Required by master plan | Not certified |

## Latest implementation commits

- `ea617b94c05b0b3b12a4433ec5d5d230b1cc81c2` — harden PCI BAR resource decoding with bounded 32/64-bit memory and I/O BAR validation, overflow/alignment checks, and regression tests.
- `bf6501856d4312b93576fba856196fdb38151911` — harden bounded PCI discovery with validated mechanism-#1 BDF/config-address construction and deterministic multi-bus scanning. This remains a runtime-neutral discovery layer and deliberately does not claim physical PCI I/O evidence.
- `d5445bc2c7a07da636584981283ba68fbecc8c48` — add the VirtIO block contract/QEMU runtime CI gate. This verifies the existing bounded VirtIO block contract under workspace tests and boots the image with a real QEMU `virtio-blk-pci` device; it deliberately does not claim guest-side persistent I/O certification.
- `5c6cc09f25ed6d421e2ecda34a6d5363faa244f9` — VirtIO block request/completion bounds hardening.
- `f1257764bb790d9047553b8733812257a6f18de5` — wire bounded PCI enumeration into the driver module.
- `c9825cb8d947ba20f1b35e9d647e0b280e8970d8` — bounded PCI config-space enumeration.
- `325f4c16ed0d823d6cd22f8ffd04574d8966dfb1` — bounded VirtIO PCI transport boundary.

## Current blockers to 100%

1. Mandatory CI gates must be green on the current revision.
2. QEMU runtime evidence must be green on the current revision.
3. The VirtIO QEMU gate must evolve from device-present boot evidence to verified guest-side read/write/flush behavior before storage certification can be claimed.
4. A platform-specific PCI config-space adapter must connect the validated BDF/config-address layer to real hardware I/O.
5. Hardware matrix, fuzz/stress and recovery evidence remain required.
6. Cryptographic trust/signing and package tooling remain incomplete where the master plan marks them open.
7. Storage, networking, userspace, AWOSA, `.asd`, `.awos`, AYUI, compatibility and App Builder release gates still require their full runtime evidence.
8. The A-C completion primitives are validation/state machinery; they do not by themselves constitute physical GDT/TSS/IDT/APIC/SMP/page-table activation or context-switch execution.
9. The driver lifecycle, PCI and VirtIO additions are bounded policy/state machinery; they do not by themselves constitute hardware-in-loop or persistent crash-injection evidence.

## Policy

Do not mark a master-plan checkbox complete merely because a contract, mock, or unit-test-only implementation exists. Certification follows the evidence requirements above.

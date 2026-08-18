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
| PCI enumeration boundary | Bounded PCI config-space enumeration validates vendor/device/header/function limits and is wired into the driver module | Implementation + unit-test evidence present; hardware enumeration pending |
| VirtIO PCI transport boundary | Identity, BAR/capability/queue/feature validation and driver-ready state | Implementation + unit-test evidence present; hardware register exercise pending |
| VirtIO block request plane | Bounded sector validation, DMA-bounded descriptor submission, queue completion and interrupt acknowledgment contract | Implementation + unit-test evidence present; persistent VirtIO device exercise pending |
| Storage GPT path | Bounded GPT scan is exercised through `BlockDevice` | Implementation + unit-test evidence present; persistent-device certification pending |
| Storage crash recovery | `JournalTxn` models prepare/commit/abort and converts non-durable states to `NeedsRecovery` | Implementation + unit-test evidence present; crash-injection evidence pending |
| Networking packet core | Ethernet/ARP/IPv4 routing and bounded UDP/TCP metadata validation | Implementation + unit-test evidence present; runtime certification pending |
| Network security policy | Allocation-free deny-by-default firewall with bounded rule capacity | Implementation + unit-test evidence present; runtime certification pending |
| Cryptographic signing | Required by master plan | Not certified |
| Hardware-in-loop matrix | Required by master plan | Not certified |
| Fuzz/stress/resource exhaustion | Required by master plan | Not certified |
| Signed reproducible release artifacts | Required by master plan | Not certified |

## Latest implementation commit

- `pending` — VirtIO block request-plane hardening: bounded block requests, DMA-safe descriptor submission, completion accounting and interrupt acknowledgment. Certification remains pending until CI and runtime evidence are green.
- `f1257764bb790d9047553b8733812257a6f18de5` — wire bounded PCI enumeration into the driver module.
- `c9825cb8d947ba20f1b35e9d647e0b280e8970d8` — bounded PCI config-space enumeration.
- `325f4c16ed0d823d6cd22f8ffd04574d8966dfb1` — bounded VirtIO PCI transport boundary.

## Current blockers to 100%

1. Mandatory CI gates must be green on the current revision.
2. QEMU runtime evidence must be green on the current revision.
3. Hardware matrix, fuzz/stress and recovery evidence remain required.
4. Cryptographic trust/signing and package tooling remain incomplete where the master plan marks them open.
5. Storage, networking, userspace, AWOSA, `.asd`, `.awos`, AYUI, compatibility and App Builder release gates still require their full runtime evidence.
6. The A-C completion primitives are validation/state machinery; they do not by themselves constitute physical GDT/TSS/IDT/APIC/SMP/page-table activation or context-switch execution.
7. The driver lifecycle, PCI and VirtIO additions are bounded policy/state machinery; they do not by themselves constitute hardware-in-loop or persistent crash-injection evidence.

## Policy

Do not mark a master-plan checkbox complete merely because a contract, mock, or unit-test-only implementation exists. Certification follows the evidence requirements above.

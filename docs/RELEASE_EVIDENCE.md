# AWE_OS Release Evidence Ledger

This document records release evidence without converting planned work into fake completion.

## Certification rule

A feature is release-certified only when implementation, tests, runtime/emulator evidence, CI, recovery/error handling, and documentation are all present. This follows `docs/AWE_OS_100_PERCENT_MASTER_PLAN.md`.

## Current evidence snapshot

| Gate | Current evidence | Certification |
|---|---|---|
| Rust workspace formatting/check/tests | CI workflow configured; current driver-supervisor revision requires a new green run | Pending latest green run |
| Clippy | CI workflow configured; current revision requires a new green run | Pending latest green run |
| UEFI loader check | CI workflow configured | Pending latest green run |
| x86_64 boot image | `boot-image.yml` builds ISO and validates ELF | Pending latest green run |
| QEMU boot smoke | Workflow checks boot/running/alive markers | Pending latest green run |
| Kernel dependency isolation | `kernel/Cargo.toml` contains only boot protocol dependency | Implemented by current source structure |
| A-C bounded runtime | `ac_boot_gate`, `ac_runtime` and `ac_completion` with deterministic tests | Implementation evidence present; runtime certification pending |
| A-C bring-up ordering | `BringupGate` covers BootInfo -> GDT/TSS -> IDT -> interrupts -> APIC -> memory -> paging -> heap -> SMP -> kernel-ready | Implementation + unit-test evidence present; hardware execution pending |
| A-C execution boundaries | Context-frame, syscall, IPC quota, capability generation, timer and trace validation are bounded and fail-closed | Implementation + unit-test evidence present; runtime certification pending |
| Driver dependency cycle rejection | `DependencyGraph` rejects self and transitive cycles | Implementation + unit-test evidence present |
| Driver lifecycle/recovery | `DriverSupervisor` enforces deterministic transitions, bounded restart budget and explicit quarantine | Implementation + unit-test evidence present; runtime certification pending |
| Storage/network product-core exercise | Product-core integration tests exist | Pending latest green run |
| Cryptographic signing | Required by master plan | Not certified |
| Hardware-in-loop matrix | Required by master plan | Not certified |
| Fuzz/stress/resource exhaustion | Required by master plan | Not certified |
| Signed reproducible release artifacts | Required by master plan | Not certified |

## Current blockers to 100%

1. Mandatory CI gates must be green on the current revision.
2. QEMU runtime evidence must be green on the current revision.
3. Hardware matrix, fuzz/stress and recovery evidence remain required.
4. Cryptographic trust/signing and package tooling remain incomplete where the master plan marks them open.
5. Storage, networking, userspace, AWOSA, `.asd`, `.awos`, AYUI, compatibility and App Builder release gates still require their full runtime evidence.
6. The A-C completion primitives are validation/state machinery; they do not by themselves constitute physical GDT/TSS/IDT/APIC/SMP/page-table activation or context-switch execution.
7. The new driver supervisor restart/quarantine policy is deterministic service-plane recovery logic; it is not hardware-in-loop evidence.

## Latest implementation commits

- `04e615eb9037c2e9e9d4b52a565ed6eccc61960f` — bounded driver restart budget and quarantine recovery policy.
- `d9adeedb5a20745afe9f6a56e2e8e7ae132ba628` — A-C completion-gate evidence ledger.

## Policy

Do not mark a master-plan checkbox complete merely because a contract, mock, or unit-test-only implementation exists. Certification follows the evidence requirements above.

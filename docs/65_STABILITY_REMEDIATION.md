# AWE_OS 65% Stability Remediation Matrix

This matrix separates defects that can make the current tree incorrect from capabilities that are intentionally still on the 65→100 roadmap. A roadmap gap is not counted as a fixed bug merely because a contract or placeholder exists.

| Priority | Area | Finding | Classification | This pass |
|---|---|---|---|---|
| P0 | x86_64 boot | GDT descriptor limit ended at the data descriptor instead of the end of the GDT | Boot correctness defect | Fixed |
| P0 | Multiboot2 handoff | Invalid magic/info was allowed to continue into `BootInfo::empty` and could produce misleading alive output | Boot validation defect | Fixed |
| P0 | Multiboot2 parser | End tag was not required before accepting the tag stream | Boot validation defect | Fixed |
| P0 | QEMU smoke test | Diagnostics were too weak to distinguish a guest reset/triple-fault from a missing serial marker | CI diagnostics defect | Fixed |
| P0 | Quality Gate | Formatting failure prevented workspace check/tests/clippy from running | CI gate blocker | Must remain green after this tree is checked |
| P1 | Kernel driver boundary | `kernel/src/drivers` exists as legacy/compatibility source, but `kernel/src/lib.rs` does not export it; concrete driver execution remains in `services/driverd` | Architecture hygiene | Audited; no runtime kernel export found |
| P1 | UEFI | Loader compile validation exists, but a real UEFI/QEMU boot exercise is still missing | Missing validation capability | Roadmap item, not silently marked fixed |
| P1 | PCI/ACPI/APIC/VirtIO | Current 65% code contains contracts/models/reference protocols; complete hardware runtime execution is not yet present | Missing implementation | Roadmap item |
| P1 | DMA/IOMMU | Full hardware DMA/IOMMU enforcement is absent | Missing implementation | Roadmap item |
| P1 | Userspace/services | The canonical seven-service roster is ahead of the currently buildable service workspace | Missing implementation | Roadmap item |
| P1 | Multi-architecture | Current automated boot certification is x86_64 only | Missing validation/implementation | Roadmap item |
| P1 | Storage/network/input/display | Full runtime stacks are not yet implemented | Missing implementation | Roadmap item |

## Gate rule

The repository must not call 65% release-certified until the P0 boot/CI gates are green. Contract-only hardware models, documentation, and metadata do not count as runtime hardware certification.

# AWE_OS Product Gap Matrix

This is the engineering map for turning the repository into a real Linux/Windows-class general-purpose OS. A file, type, trait, specification or registry entry is **not** counted as a finished feature until it is exercised by a build/test/boot gate.

## Status legend

- **Implemented** — working code exists and is covered by a meaningful validation path.
- **Foundation** — real reusable code exists, but the end-to-end product path is incomplete.
- **Planned** — architecture/docs exist or the subsystem is named, but implementation is missing.
- **Missing** — no usable implementation is present.

## 1. Boot and platform

| Area | Status | Product gate still required |
|---|---|---|
| AWE loader identity/architecture validation | Foundation | real image boot certification |
| BootInfo ABI | Foundation | ABI compatibility tests |
| ELF/image validation | Foundation | load-and-execute test |
| BIOS entry | Foundation | stage-2 disk read + mode transition |
| UEFI | Foundation | real UEFI loader + memory map |
| Page-table/CR3 handoff | Planned | activate paging from loader |
| Secure/measured boot | Planned | cryptographic release-image verification |
| Anti-rollback | Foundation | persistent monotonic version source |
| Recovery boot | Planned | failed-update recovery test |

## 2. CPU and kernel core

| Area | Status | Product gate still required |
|---|---|---|
| x86_64 entry/IDT/ISR | Foundation | full exception + interrupt integration |
| GDT/TSS | Planned | privilege-transition tests |
| APIC/IOAPIC | Planned | SMP interrupt tests |
| SMP/multicore | Planned | multi-core QEMU/physical tests |
| Timer | Foundation | scheduler-driven timer interrupts |
| Physical allocator | Foundation | stress/fragmentation tests |
| Virtual memory/paging | Foundation | user address-space tests |
| Kernel heap | Planned | allocator integration + stress |
| Panic/fault handling | Foundation | deterministic crash/recovery tests |

## 3. Execution model

| Area | Status | Product gate still required |
|---|---|---|
| Process descriptors/states | Foundation | real process creation |
| Context switching | Foundation | runnable process execution |
| Scheduler | Foundation | timer-preemption + fairness tests |
| Syscall ABI/dispatch | Foundation | CPU trap entry + user transition |
| IPC | Foundation | integrated channel/endpoint syscalls |
| Capabilities | Foundation | capability enforcement on real resources |
| User/kernel isolation | Planned | ring/user address-space isolation |
| Signals/events | Missing | design + implementation |

## 4. Drivers and hardware

The repository already contains a substantial driver-management architecture: contracts, registry, bus/HAL abstractions, VirtIO negotiation, provenance/compatibility metadata and Linux-driver lifecycle/recovery components. The main missing part is **real hardware transport and device-driver execution**.

| Area | Status | Product gate still required |
|---|---|---|
| Driver HAL/contracts | Foundation | hardware integration |
| Device registry | Foundation | PCI-discovered devices |
| VirtIO negotiation | Foundation | actual VirtIO PCI transport |
| VirtIO block | Planned | read/write + QEMU test |
| VirtIO network | Planned | packet TX/RX + network tests |
| VirtIO console/input | Planned | device I/O tests |
| PCI/PCIe | Planned | enumeration/config-space tests |
| ACPI | Planned | table parser + power/CPU discovery |
| NVMe | Planned | block I/O + filesystem integration |
| AHCI/SATA | Planned | block I/O tests |
| USB/xHCI | Planned | controller + enumeration |
| HID keyboard/mouse | Planned | input event pipeline |
| Ethernet | Planned | real NIC + QEMU coverage |
| Wi-Fi | Planned | firmware + at least one reference chipset |
| GPU/framebuffer | Planned | framebuffer first, GPU acceleration later |
| Audio | Planned | at least one reference device |
| IOMMU/SMMU | Planned | DMA isolation tests |
| Linux driver compatibility | Architecture | actual ABI/runtime compatibility is a separate project |
| Windows driver compatibility | Architecture | documented-device/native adapter strategy |
| Android vendor compatibility | Architecture | vendor HAL/device adapter strategy |

## 5. Storage and networking

| Area | Status | Product gate still required |
|---|---|---|
| Block-device abstraction | Planned | device-backed implementation |
| VFS | Planned | mount/open/read/write/stat |
| AWEFS | Planned | persistent on-disk format + fsck/recovery |
| FAT/ESP support | Planned | boot/update/install path |
| Partitioning GPT | Missing | parser + safe writer |
| NVMe/AHCI filesystem path | Missing | end-to-end disk boot/install |
| TCP/IP | Missing | sockets + DHCP/static config |
| UDP/DNS | Missing | resolver + datagram API |
| IPv6 | Missing | later production gate |
| TLS/secure networking | Missing | userspace/runtime integration |

## 6. System services and userspace

| Area | Status | Product gate still required |
|---|---|---|
| Init/service manager | Planned | PID 1 equivalent + dependency graph |
| Device manager | Foundation | connect registry to userspace/services |
| Time service | Foundation | hardware clock/timer source |
| Kernel logging | Foundation | serial/framebuffer sink + userspace reader |
| Shell/terminal | Missing | interactive TTY/PTY system |
| User accounts | Missing | identity/permission store |
| Permissions | Foundation | userspace enforcement |
| Process supervisor | Missing | restart/limits/service lifecycle |
| Crash reporting | Missing | persistent diagnostics |

## 7. AWOSA / application platform

| Area | Status | Product gate still required |
|---|---|---|
| Manifest model | Architecture | parser + validation |
| Package format | Architecture | package manager + repository |
| Package signatures | Architecture | key trust + update verification |
| Sandbox | Architecture | enforceable isolation |
| Resource budgets | Foundation | runtime enforcement |
| AWE Capsule | Foundation | real executable launcher |
| App lifecycle | Missing | install/start/stop/update/uninstall |
| Developer SDK | Missing | stable app ABI/tooling |

## 8. Desktop

| Area | Status | Product gate still required |
|---|---|---|
| Framebuffer | Planned | display driver |
| GPU acceleration | Missing | graphics API + reference GPU |
| Compositor/window manager | Missing | surface protocol + compositor |
| AYUI toolkit | Architecture | actual widget/rendering system |
| File manager | Missing | filesystem/userspace integration |
| Settings | Missing | configuration service |
| System monitor | Missing | metrics/process/device APIs |
| Notifications | Missing | desktop service |
| Clipboard | Missing | IPC/desktop service |
| Accessibility | Missing | accessibility API |

## 9. Compatibility

Compatibility must be treated as separate product layers rather than claiming that foreign kernel modules can simply be loaded into CellKernel.

- **Linux/POSIX:** syscall/libc compatibility first; later selected Linux application/runtime compatibility.
- **Windows:** Win32/NT API compatibility or a VM/container strategy; native `.sys` execution is a much larger ABI/security project.
- **Android:** Android userspace/runtime and selected HAL compatibility; vendor blobs require strict licensing and isolation handling.

## 10. Production engineering

| Area | Status | Product gate |
|---|---|---|
| Reproducible builds | Foundation | byte/release reproducibility job |
| CI formatting/build/test/Clippy | Implemented | keep green |
| QEMU boot certification | Planned | boot + smoke-test matrix |
| Hardware matrix | Planned | certified reference systems |
| Fuzzing | Planned | parser/ABI/driver fuzz targets |
| Sanitizers/UB checks | Planned | host-side + supported kernel checks |
| Benchmarks | Planned | scheduler/memory/I/O/network baseline |
| Release signing | Planned | key policy + verification in boot path |
| OTA/update | Missing | atomic A/B update + rollback |
| Security response process | Foundation | advisories + reproducible security tests |

## Priority order

1. **Bootable x86_64 path** — UEFI + page tables + deterministic kernel entry.
2. **Memory/interrupt/SMP core** — make CellKernel genuinely execute and schedule.
3. **User boundary** — syscall trap, address spaces, IPC, capability enforcement.
4. **PCI + VirtIO** — create a real device graph and testable storage/network paths.
5. **Storage** — GPT + block layer + VFS + AWEFS/ESP support.
6. **Networking** — Ethernet/VirtIO-net + UDP/TCP/DNS.
7. **Init + userspace** — service manager, shell, package/runtime primitives.
8. **Desktop** — framebuffer → compositor → AYUI.
9. **Compatibility** — POSIX/Linux first; Windows/Android as isolated compatibility layers.
10. **Production** — signed releases, reproducibility, fuzzing, recovery and hardware certification.

## Engineering rule

The project should increase its product percentage only when a subsystem moves from architecture/foundation to **verified execution**. This prevents documentation and placeholder modules from inflating the readiness score.

# AWE_OS 100% Product Master Plan v2

> **Goal:** ship AWE_OS as a real, independently bootable, modular and security-first operating-system product.
>
> **Rule:** 100% is a release certification state, not a documentation state. A feature receives progress credit only after implementation + tests + runtime/emulator evidence + CI + recovery/error handling + documentation.

## 1. Product definition

AWE_OS 1.0 is complete only when a clean build produces signed, reproducible boot artifacts; the system boots reliably on the supported hardware/emulator matrix; CellKernel, userspace, storage, networking, native applications and the desktop operate together; privileged components are isolated and capability-controlled; updates and recovery are safe; and all mandatory CI gates are green.

## 2. Architecture invariants

- CellKernel stays small, deterministic and privileged.
- Drivers, storage, networking, UI and compatibility layers remain modular services/components.
- No compatibility layer is allowed to become a hidden kernel dependency.
- Capability checks happen at the service boundary and again at sensitive resource boundaries.
- Every untrusted parser/package/driver boundary is bounded and fail-closed.
- Fixed-capacity or explicitly budgeted structures are preferred in privileged code.
- Public ABIs are versioned; incompatible changes require an explicit migration policy.
- Performance claims require repeatable benchmarks.
- Recovery paths are part of the feature, not a later add-on.

## 3. Product scorecard — 100 points

| Area | Weight | Release gate |
|---|---:|---|
| Boot/platform | 10 | UEFI/QEMU boot, CPU/interrupt/memory validation |
| CellKernel | 15 | scheduler, processes, VM, syscalls, IPC, capabilities |
| Drivers/device model | 15 | device discovery, isolation, DMA/IOMMU boundary, recovery |
| Storage/filesystem | 8 | VFS + native filesystem + recovery + persistent tests |
| Networking | 7 | Ethernet/IP/TCP/UDP/DNS + policy + integration tests |
| Userspace/services | 8 | init/service manager + identity + core services |
| AWOSA/native ABI | 5 | stable runtime and SDK contract |
| `.asd` driver ecosystem | 5 | signed package/install/upgrade/rollback/recovery |
| `.awos` app ecosystem | 5 | signed package/install/run/update/remove + dependencies |
| Compatibility | 10 | tested Linux/Windows/Android subsets, explicitly bounded |
| AYUI desktop | 5 | compositor/windowing/input/display + core UX |
| App Builder/SDK | 3 | create/build/test/debug/package workflows |
| Security/update/recovery | 3 | trust chain, sandboxing, update rollback, recovery |
| Testing/hardware/release | 6 | QEMU + hardware matrix + fuzz/stress + reproducible release |
| **Total** | **100** | **all mandatory gates green** |

## 4. Stage A — Engineering foundation

- [ ] Pin Rust/toolchains and build inputs.
- [ ] Make `fmt`, `check`, tests and Clippy warning-clean.
- [ ] Split CI into fast, kernel, boot, device, runtime, security and release gates.
- [ ] Publish reproducible build metadata and artifact manifests.
- [ ] Add architecture-contract tests preventing forbidden module dependencies.
- [ ] Keep all privileged modules covered by deterministic unit tests.

## 5. Stage B — Boot and hardware bring-up

- [ ] Version and validate BootInfo ABI.
- [ ] Complete GDT/TSS/IDT, exception and timer paths.
- [ ] Complete APIC/IOAPIC routing.
- [ ] Bring up SMP and CPU topology.
- [ ] Complete physical frame discovery/allocation.
- [ ] Complete virtual memory, page tables and kernel heap.
- [ ] Add robust panic/diagnostic path.
- [ ] Require QEMU boot smoke tests in CI.

## 6. Stage C — CellKernel execution core

- [ ] Preemptive priority scheduler and accounting.
- [ ] Process/thread lifecycle and context switching.
- [ ] User/kernel address-space separation.
- [ ] Syscall ABI and argument validation.
- [ ] Bounded IPC with backpressure and quotas.
- [ ] Events/notifications and synchronization primitives.
- [ ] Capability lifecycle, revocation and audit/provenance.
- [ ] Monotonic time, timers and resource limits.
- [ ] Kernel tracing and structured diagnostics.

## 7. Stage D — Modular device platform

Native driver lifecycle:

`discover -> identify -> probe -> bind -> initialize -> run -> suspend -> resume -> stop -> remove -> recover`

- [ ] Stable driver ABI and manifest.
- [ ] Device identity and dependency graph with cycle rejection.
- [ ] Explicit MMIO/PIO/DMA/interrupt ownership.
- [ ] DMA/IOMMU enforcement boundary.
- [ ] Driver health monitoring and bounded restart policy.
- [ ] Signed-driver trust and ABI compatibility.
- [ ] PCI/PCIe, ACPI, APIC/IOAPIC, VirtIO.
- [ ] NVMe/AHCI, USB/xHCI, HID, display, Ethernet, Wi-Fi, Bluetooth, audio, power/thermal and RTC.

### `.asd` driver package

- [x] Versioned canonical container and bounded parser.
- [x] Manifest, capabilities, ABI and architecture fields.
- [x] Bounded manifest/payload/signature lengths with overflow-safe total-size validation.
- [ ] Strong hashes and cryptographic signatures.
- [ ] `asdpack` / `asddump` tooling.
- [x] Install/active/staged/failed/quarantined lifecycle contract.
- [ ] Full install, verify, upgrade, rollback, uninstall and recovery tooling.
- [ ] Tamper/malformed-package fuzz tests.

## 8. Stage E — Storage

- [ ] VFS and native AWE filesystem.
- [ ] Permissions/capabilities.
- [ ] Journaling and crash recovery.
- [ ] Block cache and storage service.
- [ ] NVMe/AHCI integration.
- [ ] Mount lifecycle and fsck/recovery tools.
- [ ] Read-only recovery environment.
- [ ] Encryption-at-rest where supported by the release design.

## 9. Stage F — Networking

- [ ] Network device service.
- [ ] Ethernet, ARP/IPv4, IPv6/ND, ICMP.
- [ ] UDP, TCP, DNS, DHCP and sockets.
- [ ] TLS integration.
- [ ] Firewall/security policy.
- [ ] Isolation/namespaces where required.
- [ ] Wi-Fi/Bluetooth integration.
- [ ] Network stress, timeout, retry and recovery tests.

## 10. Stage G — Userspace and services

- [ ] User-space loader.
- [x] Init/service manager.
- [x] User/group identity model.
- [ ] Device, filesystem and network managers.
- [x] Update manager contract with bounded A/B state machine.
- [ ] Logging/diagnostics.
- [ ] Security policy service.
- [ ] Crash reporting and recovery.

Canonical services must have versioned contracts, bounded resources and explicit capability admission.

## 11. Stage H — AWOSA native runtime

- [ ] Stable runtime ABI.
- [ ] Process/memory/filesystem/network/UI/device/IPC APIs.
- [ ] Capability and permission APIs.
- [ ] Async/concurrency primitives.
- [x] Runtime version negotiation.
- [ ] SDK, headers and bindings.
- [ ] ABI compatibility tests.

## 12. Stage I — `.awos` native application platform

- [x] Versioned executable/package format and bounded parser.
- [x] Manifest/resource/dependency counters and bounded package sections.
- [x] Entry-point, signature-length and total-size validation before admission.
- [ ] Publisher identity, cryptographic signatures and sandbox policy integration.
- [ ] Update/rollback metadata integration.
- [ ] `awospack` and verification tools.
- [x] Installed/running/staged/failed/quarantined/removed lifecycle contract.
- [ ] Repository/index and dependency resolution.
- [ ] Package integrity and permission integration tests.

## 13. Stage J — AYUI desktop

- [ ] Display server/compositor.
- [ ] Window manager and input system.
- [ ] GPU acceleration path.
- [ ] Fonts/text/accessibility.
- [ ] Themes, notifications, clipboard and drag/drop.
- [ ] Multi-monitor.
- [ ] Settings, Files, Terminal, System Monitor and launcher.

## 14. Stage K — Compatibility, always isolated

### Linux
- [ ] POSIX/Linux ABI subset.
- [ ] ELF loader boundary.
- [ ] libc/process/thread/signal mappings.
- [ ] filesystem/network/graphics integration.
- [ ] tested application matrix.

### Windows
- [ ] PE/Win32 compatibility strategy.
- [ ] registry/config boundary.
- [ ] graphics/audio/input mappings.
- [ ] isolation and compatibility matrix.

### Android
- [ ] package/runtime boundary.
- [ ] Binder-compatible IPC where required.
- [ ] API/permission/graphics/input/audio mappings.
- [ ] compatibility matrix.

Compatibility must never be described as native support. Native `.asd` drivers remain the preferred trusted driver path.

## 15. Stage L — AWEOSA App Builder and SDK

- [ ] CLI project generator.
- [ ] GUI project creator.
- [ ] AWOSA SDK integration.
- [ ] editor, syntax highlighting and build/run/debug.
- [ ] simulator/emulator integration.
- [ ] UI designer and component library.
- [ ] assets and package/signing assistant.
- [ ] capability editor and test runner.
- [ ] profiler/log viewer.
- [ ] `.awos` exporter.
- [ ] safer `.asd` driver-development mode.
- [ ] desktop/CLI/service/driver/compatibility templates.

## 16. Stage M — First-party ecosystem

- [ ] AWE Terminal
- [ ] AWE Files
- [ ] AWE Settings
- [ ] AWE Browser
- [ ] AWE Text Editor
- [ ] AWE System Monitor
- [ ] AWE Package Center
- [ ] AWE Developer Studio / App Builder
- [ ] AWE Update Center
- [ ] AWE Recovery

## 17. Stage N — Security hardening

- [ ] Secure-boot/trust-chain integration.
- [ ] Signed boot, driver and application artifacts.
- [ ] Capability security and least privilege.
- [ ] Sandboxing and fault isolation.
- [ ] Memory-safety review of privileged boundaries.
- [ ] Package/parser fuzzing.
- [ ] Syscall fuzzing.
- [ ] Driver fault injection.
- [x] Update generation monotonicity and downgrade rejection contract.
- [ ] Key/secret handling policy.
- [ ] Security incident and recovery procedures.

## 18. Stage O — Update, recovery and lifecycle

- [x] Atomic A/B update state machine contract.
- [x] A/B rollback selection and failed-boot recovery contract.
- [ ] Driver/application rollback integration.
- [ ] Recovery boot target.
- [ ] Offline repair.
- [ ] Backup/restore.
- [ ] Release channels and signed manifests.
- [ ] Reproducible release builds.

## 19. Stage P — Multi-architecture

Production order:

1. x86_64
2. ARM64
3. RISC-V 64

An architecture is complete only after boot, memory, interrupts, scheduling, drivers, userspace and CI validation all pass.

## 20. Stage Q — Evidence and release certification

Every release candidate must pass:

- [ ] clean reproducible build;
- [ ] unit/integration tests;
- [ ] QEMU boot and device exercise;
- [ ] storage/network/input/display runtime exercise;
- [ ] hardware-in-the-loop matrix for supported devices;
- [ ] crash/recovery tests;
- [ ] suspend/resume where supported;
- [ ] fuzz/stress/resource-exhaustion tests;
- [ ] performance regression budget;
- [ ] security gates;
- [ ] signed release artifacts;
- [ ] no unresolved critical/high issue.

## 21. Progress accounting

The percentage is computed from validated evidence, never from line count, file count, mock interfaces or documentation.

A milestone can move from implementation-complete to certified only after its required evidence gate is green. If a later regression breaks a gate, the affected milestone is automatically considered uncertified again.

### 90% implementation checkpoint

The current implementation checkpoint adds concrete product-core contracts for `.asd`, `.awos`, and atomic A/B update/recovery. These are deliberately marked as implementation-level progress; the evidence gates above remain mandatory before certification credit is granted.

## 22. Execution order

`CI foundation -> boot -> kernel execution -> memory/process/syscalls -> device model -> PCI/ACPI/APIC -> VirtIO -> storage -> filesystem -> networking -> userspace -> AWOSA -> .awos -> .asd -> AYUI -> native apps -> compatibility -> App Builder -> security hardening -> update/recovery -> multi-architecture -> hardware matrix -> release 1.0`

This order is intentionally modular: UI and ecosystem work must not hide instability in the privileged core.

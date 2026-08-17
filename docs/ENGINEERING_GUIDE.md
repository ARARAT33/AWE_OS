# AWE_OS Engineering Guide

AWE_OS is a Rust-first operating-system project built around a small privileged CellKernel, a dedicated AWE boot path, capability-controlled services, and architecture-specific hardware support.

## 1. Source-of-truth map

| Area | Location | Responsibility |
|---|---|---|
| Workspace | `Cargo.toml` | Rust workspace and release profiles |
| Boot protocol | `boot/protocol` | Stable loader/kernel handoff contracts |
| AWE loader | `boot/aweloader` | Image identity, architecture, memory and security validation |
| Kernel | `kernel` | CellKernel core primitives |
| Kernel binary | `kernel-bin` | Kernel executable/integration entry |
| AWOSA tooling | `tools/aweosa-builder` | Native application/package tooling |
| CI | `.github/workflows` | Formatting, tests, linting, security and boot-image gates |
| Architecture | `ARCHITECTURE.md` | Long-term system design |
| Readiness | `PRODUCT_READINESS.md` | Evidence-based product gates |

## 2. Core invariants

1. **Fail closed:** invalid boot metadata, unknown hardware, invalid capabilities and malformed syscall inputs must be rejected.
2. **No implicit privilege:** every privileged operation has an explicit authorization path.
3. **Bounded kernel state:** early boot registries, journals, queues and token buckets must have deterministic capacity.
4. **Architecture isolation:** architecture-specific instructions stay behind `kernel/src/arch` or an equivalent HAL boundary.
5. **No foreign ABI in CellKernel:** Linux/Windows/Android compatibility belongs in isolated compatibility services.
6. **Deterministic transitions:** boot phases, scheduler primitives and security decisions must be testable without timing assumptions.
7. **Reproducible evidence:** a feature is not considered product-ready until CI or an explicit hardware/QEMU test demonstrates it.

## 3. Development loop

```text
Design invariant
      ↓
Implement the smallest no_std primitive
      ↓
Add unit/property-style tests where possible
      ↓
cargo fmt --all -- --check
      ↓
cargo check --workspace
      ↓
cargo test --workspace --no-fail-fast
      ↓
cargo clippy --workspace --all-targets -- -D warnings
      ↓
QEMU / real-hardware validation
      ↓
Update PRODUCT_READINESS.md with evidence
```

## 4. Kernel layering

```text
Firmware / UEFI
      ↓
AWE Loader
      ↓
Boot Protocol
      ↓
CellKernel entry
      ├── architecture + CPU
      ├── memory
      ├── interrupts
      ├── scheduler
      ├── process + IPC
      ├── syscall boundary
      ├── security / capabilities
      └── driver HAL
            ↓
      storage / network / input / display
            ↓
      services + AWOSA runtime
            ↓
      AYUI / terminal / applications
```

## 5. Current implementation boundary

The repository already contains substantial kernel foundations: authorization and capability primitives, deterministic scheduling pieces, typed memory addresses and early mapping, interrupt/CPU primitives, driver contracts, VirtIO negotiation, syscall validation, bootloader validation and CI security gates. These foundations should be treated as contracts and extended rather than duplicated.

The largest remaining product risks are end-to-end execution: complete x86_64 UEFI boot, page-table activation and handoff, heap activation, real interrupt/timer execution, scheduler context switching, user process isolation, IPC transport, PCI/ACPI discovery, storage/network/input/display drivers, DMA isolation and automated QEMU certification.

## 6. Rules for new kernel code

- Prefer fixed-size structures during early boot.
- Return explicit error types instead of silently recovering from security failures.
- Keep `unsafe` blocks small and document the hardware invariant they rely on.
- Avoid allocation in boot-critical code.
- Keep public ABI structures `#[repr(C)]` when crossing component boundaries.
- Add regression tests for every security or parser bug.
- Never mark a roadmap item complete merely because a type or placeholder exists.

## 7. Release evidence

Every product milestone should record:

- commit SHA;
- target architecture;
- toolchain version;
- exact CI checks passed;
- QEMU configuration, when applicable;
- hardware model, when applicable;
- known limitations;
- rollback/recovery behavior.

This keeps AWE_OS measurable as an operating-system product instead of a documentation-only architecture.

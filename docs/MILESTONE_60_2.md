# AWE_OS 60.2 — Kernel/Service Contract Freeze

## Status

**Milestone: 60.2 — implemented in the repository.**

60.2 is a contract milestone, not a claim that AWE_OS is 60.2% closer to release. The gate establishes a stable, typed and versioned boundary between CellKernel and the user-space service plane.

## Frozen ABI

- CellKernel ABI: `1.2`
- Driver service ABI: `1.2`
- Native application service ABI: `1.2`
- ABI major changes are breaking.
- Services may target an older minor ABI when the kernel advertises a newer compatible minor version.

## Capability model

The kernel contract exposes explicit capabilities for:

- memory
- interrupts
- scheduling
- processes
- IPC
- syscalls
- security
- device grants
- DMA
- shared memory

Services cannot gain capabilities implicitly. A service contract is accepted only when the kernel satisfies the service's major/minor ABI and required capability set.

## Service IDs

The stable service namespace reserves IDs for:

- `driverd`
- `appd`
- `asappd`
- `ayuid`
- `aweterminald`
- `awebusd`
- `aweupdated`

Only the service contract is frozen at this milestone; later milestones provide the corresponding full runtimes.

## IPC boundary

The kernel IPC layer now exposes typed service channels and stable opcodes for:

- Hello
- Ping
- Start
- Stop
- Reset
- Query
- Event
- Handoff

The underlying mailbox remains bounded and allocation-free during bootstrap.

## 60.2 acceptance criteria

- [x] ABI major/minor constants are centralized.
- [x] Service IDs are stable and typed.
- [x] Required capabilities are explicit.
- [x] Service admission is fail-closed on missing capabilities.
- [x] Major version mismatch is rejected.
- [x] Older compatible minor versions are accepted.
- [x] IPC service channels/opcodes are typed.
- [x] driverd is aligned to service ABI 1.2.
- [x] appd is aligned to service ABI 1.2.
- [x] Unit tests cover capability, version and IPC invariants.

## Non-goals

60.2 does not claim that PCI, VirtIO, ASAD, ASAP, ASAPP, AYUI, live updates, or the complete desktop environment are implemented. Those belong to later milestones in the 60→100% production plan.

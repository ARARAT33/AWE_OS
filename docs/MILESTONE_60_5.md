# AWE_OS 60.5 — System Service / Process Model Freeze

## Status

**Code milestone: COMPLETE.**

60.5 freezes the process-level contract used by AWE_OS system services. It does not place service implementations back into CellKernel.

## Scope

The kernel now exposes a small, deterministic service-process model:

- explicit `ServiceId` ownership;
- explicit `ProcessId` association;
- service class classification;
- explicit lifecycle state;
- bounded CPU/memory/IPC resource budget;
- capability set attached to the service process;
- fail-closed capability admission;
- no driver/application implementation inside the kernel.

## Service classes

- `System` — core OS services;
- `Hardware` — `driverd` and hardware-facing services;
- `Application` — `appd` and native application services;
- `Interface` — AYUI and terminal-facing services;
- `Compatibility` — Linux/Windows/Android/macOS compatibility services;
- `Update` — AWEUpdate and recovery services.

## Lifecycle

```text
DECLARED → STARTING → RUNNING → STOPPING
                   ├────────────→ FAILED
                   └────────────→ QUARANTINED
```

Only the process/scheduler primitives belong to CellKernel. Service supervisors remain in userspace.

## Acceptance criteria

- [x] Service descriptors are fixed-layout and allocation-free.
- [x] Every service has an explicit process owner.
- [x] Every service carries a capability set.
- [x] Every service carries CPU/memory/IPC budgets.
- [x] Lifecycle transitions are explicit and deterministic.
- [x] Capability admission fails closed.
- [x] Driver code remains outside CellKernel.
- [x] App code remains outside CellKernel.
- [x] Unit tests cover lifecycle and capability rejection.

## Next gate: 60.6–60.8

The next block implements the execution-side IPC/service boundary: capability handles, service registration/handshake, shared-memory channels and deterministic asynchronous request/event transport.

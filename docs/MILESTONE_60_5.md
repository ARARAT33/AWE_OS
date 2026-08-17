# AWE_OS 60.5 — Architecture Freeze: Process + Service Ownership

## Status

**Code milestone: COMPLETE.**

60.5 is the exact midpoint gate inside the **60–61% Architecture Freeze** block from the AWE_OS master plan. It freezes the process/service ownership model without moving driver, application, UI, terminal, compatibility, or update implementations into CellKernel.

## What 60.5 freezes

- Every system service has a stable `ServiceId`.
- Every service is owned by exactly one `ProcessId` at runtime.
- Service class is explicit: Hardware, Application, Interface, Compatibility, System, or Update.
- Service lifecycle is explicit and deterministic.
- Service capabilities are explicit and fail closed.
- CPU, memory and IPC budgets are explicit and bounded.
- Kernel service metadata is fixed-capacity and allocation-free during bootstrap.
- Driver/application implementations remain outside CellKernel.
- A canonical seven-service roster is defined for the platform boundary.

## Canonical service roster

| Service | Class | Role |
|---|---|---|
| `driverd` | Hardware | isolated driver plane |
| `appd` | Application | native application/package plane |
| `asappd` | Application | ASAPP build/development service |
| `ayuid` | Interface | AYUI UI service |
| `aweterminald` | Interface | AWETerminal service |
| `awebusd` | System | system bus/service coordination |
| `aweupdated` | Update | update/recovery service |

## Lifecycle

```text
DECLARED → STARTING → RUNNING → STOPPING
                   ├────────────→ FAILED
                   └────────────→ QUARANTINED
```

Only process/scheduler primitives and the minimal service metadata model belong to CellKernel. Service supervisors and implementations remain in user space.

## Resource model

Every service process carries:

- CPU budget;
- memory budget;
- IPC message budget;
- capability set;
- process owner identity.

The bootstrap model uses bounded quotas; later runtime negotiation may adjust quotas without changing the service ABI.

## Acceptance criteria

- [x] `ServiceDescriptor` is fixed-layout and allocation-free.
- [x] `ServiceRegistry<N>` is bounded and duplicate-safe.
- [x] Canonical service IDs/classes are frozen.
- [x] Service/process ownership is explicit.
- [x] Lifecycle states are explicit and deterministic.
- [x] Capability admission fails closed.
- [x] CPU/memory/IPC quotas are carried with the service descriptor.
- [x] Driver code remains outside CellKernel.
- [x] Application code remains outside CellKernel.
- [x] Tests cover lifecycle, registry bounds/duplicates, roster stability and capability rejection.

## Validation rule

This milestone is complete as an engineering/code gate. The headline product percentage remains **60%** until the next substantive product gate in the master plan is implemented and validated.

## Next planned gate

**60.6–60.8:** execution-side IPC/service boundary: capability handles, service registration/handshake, shared-memory channels, deterministic asynchronous request/event transport.

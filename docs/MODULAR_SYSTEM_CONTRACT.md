# AWE_OS Modular System Contract

This document turns AWE_OS modularity into enforceable engineering rules.

## Boundary map

```text
boot -> kernel -> ipc/capabilities -> services -> drivers/storage/network
                                             |
                                             +-> AWOSA -> .awos -> AYUI/apps
                                             +-> compatibility modules
```

## Rules

1. **Kernel minimality:** CellKernel may own CPU, memory, interrupts, scheduling, syscall entry, IPC primitives and capability enforcement. It must not contain application, UI, filesystem policy or compatibility implementations.
2. **Driver isolation:** concrete hardware drivers live outside CellKernel whenever the hardware boundary permits it. Driver access is mediated by authenticated service endpoints and explicit resource grants.
3. **Service contracts:** every privileged service has a versioned ABI, explicit capability requirements, bounded resource budgets and lifecycle states.
4. **Fail-closed boundaries:** malformed ABI messages, package manifests, device identities, addresses, sizes and capabilities are rejected without partial activation.
5. **No hidden coupling:** optional modules must not become compile-time requirements of unrelated modules. Compatibility layers must remain replaceable.
6. **Versioned interfaces:** breaking changes require an ABI version increment and an explicit compatibility/migration path.
7. **Determinism:** privileged validation and admission decisions must not depend on unbounded allocation or uncontrolled global state.
8. **Recovery:** every long-lived service/driver defines failure, restart and rollback behavior before it is considered production-ready.
9. **Observability:** security-sensitive lifecycle decisions produce bounded, structured diagnostics/provenance suitable for CI and incident analysis.
10. **Evidence:** a module is product-complete only after code, tests, runtime/emulator evidence, CI, documentation and recovery behavior all pass.

## Required module contract

Each production module should expose:

- `abi_version`
- `capabilities`
- `resource_budget`
- `lifecycle_state`
- `health_state`
- `error/recovery policy`
- `test/evidence identifier`

## Release invariant

No feature may be marked 100% merely because an interface or mock exists. AWE_OS 100% means the integrated product works through the complete boot -> kernel -> service -> device/storage/network -> userspace -> application path on the supported validation matrix.

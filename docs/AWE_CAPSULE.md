# AWE Capsule — Native Application Model

AWE Capsule is the proposed native application contract for AWE_OS. It combines executable code, capabilities, provenance, resource limits and recovery metadata into one verifiable unit.

## Capsule fields

```text
Header
Identity / signer
Executable segments
Capability manifest
Resource budget
Dependency graph
Provenance metadata
Update channel
Recovery metadata
Integrity measurements
```

## Design goals

1. **Least privilege by construction** — applications receive declared capabilities rather than ambient authority.
2. **Predictable resources** — CPU, memory, storage and IPC budgets are explicit policy inputs.
3. **Verifiable updates** — package identity and integrity are checked before activation.
4. **Safe rollback** — updates can be staged and reversed using the platform recovery model.
5. **Auditable execution** — security-sensitive operations can be correlated with capsule identity and provenance.

The capsule format is a platform design specification and is not yet the final binary ABI.

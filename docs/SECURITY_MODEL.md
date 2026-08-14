# AWEOS Security Model

Security is a kernel architecture property, not an optional application.

## Core rules

- privileged code is minimized
- user processes cannot directly access arbitrary physical memory
- executable memory is controlled
- capabilities are explicit and revocable
- IPC endpoints are capability-gated
- drivers have least privilege
- malformed device input must not crash the kernel
- watchdogs and bounded operations prevent permanent stalls
- native applications are signed and sandboxed by default
- compatibility environments are isolated from native services

## Threat model

AWEOS must assume malicious applications, compromised documents, hostile network traffic, malformed USB/PCI devices and buggy third-party drivers. The design therefore favors isolation, validation, recovery and defense in depth.

## Performance

Security mechanisms should use hardware support where available and avoid unnecessary copies. Shared-memory IPC is allowed only with ownership, lifetime and synchronization rules that can be verified.

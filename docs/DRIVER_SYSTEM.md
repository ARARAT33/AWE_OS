# AWE_OS Driver Intelligence System

## Goal

AWE_OS uses one driver architecture for native AWE drivers and reviewed compatibility adapters derived from Linux, Android, Windows, BSD and vendor specifications. Foreign kernel ABIs are never executed directly inside CellKernel.

## Offline + online database

The offline database is a signed, versioned hardware index shipped with AWE_OS. It contains hardware identifiers, compatible driver manifests, ABI versions, required capabilities, firmware requirements, verification state and known-good versions.

The online database is an update channel. Downloads enter an isolated staging area, are authenticated and integrity-checked, then undergo manifest, provenance, compatibility and policy validation before becoming installable. The previous known-good database and driver remain available for rollback.

## Driver learning

AWE_OS records bounded driver experience: device identity, driver identity, probe attempts, successes, failures and last outcome. This is **telemetry/state for deterministic selection**, not unconstrained self-modifying kernel code.

A driver is not automatically promoted merely because it succeeded once. The initial stability rule requires multiple successful observations and more successes than failures. Production policy can require hardware certification and signatures before automatic activation.

## Closed-source / proprietary drivers

AWE_OS does not copy proprietary driver source or binaries without permission. For proprietary devices, compatibility can be implemented through documented device protocols, vendor firmware interfaces, standardized buses, published APIs, or a user-space adapter where legally and technically appropriate.

## Safety model

- Driver MMIO/DMA/IRQ access is described by a DeviceContract.
- Unverified manifests cannot bind.
- Unknown devices fail closed.
- Driver crashes are intended to be isolated from the trusted kernel.
- IOMMU/DMA isolation is a release requirement for untrusted/high-risk drivers.
- Updates are rollback-safe.

## Compatibility target

The database schema is designed to grow toward broad PC, server, laptop, ARM, embedded and VM coverage. It does not claim impossible universal compatibility before a device has been tested.

# AWE_OS Documentation Index

This directory is the engineering documentation surface for AWE_OS. Code is authoritative for implemented behavior; these documents describe contracts, design, validation and roadmap.

## Start here

1. [ENGINEERING_GUIDE.md](ENGINEERING_GUIDE.md) — how the repository is structured and how changes are validated.
2. [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — implementation status snapshot.
3. [PRODUCT_READINESS.md](../PRODUCT_READINESS.md) — product gates and evidence standard.
4. [AWE_OS_100_PERCENT_MASTER_PLAN.md](AWE_OS_100_PERCENT_MASTER_PLAN.md) — complete roadmap to 1.0.
5. [MILESTONE_60_2.md](MILESTONE_60_2.md) — implemented 60.2 CellKernel/service contract freeze.
6. [MILESTONE_60_5.md](MILESTONE_60_5.md) — implemented 60.5 system service/process model freeze.
7. [MILESTONE_61_0.md](MILESTONE_61_0.md) — implemented 61.0 Architecture Freeze and service transport boundary.
8. [MILESTONE_61_5.md](MILESTONE_61_5.md) — implemented 61.5 canonical device-boundary freeze.
9. [MILESTONE_62_0.md](MILESTONE_62_0.md) — implemented 62.0 driver capability/resource integration.
10. [MILESTONE_62_5.md](MILESTONE_62_5.md) — implemented 62.5 native driver manifest/lifecycle freeze.
11. [MILESTONE_63_0.md](MILESTONE_63_0.md) — implemented 63.0 dependency/ownership/health gate.
12. [MILESTONE_64_0.md](MILESTONE_64_0.md) — implemented 64.0 hardware-access boundary.
13. [MILESTONE_65_0.md](MILESTONE_65_0.md) — implemented 65.0 hardware execution checkpoint; release certification remains evidence-driven.

## Boot and kernel

- [ABI.md](ABI.md)
- [BOOT_ARCHITECTURE.md](BOOT_ARCHITECTURE.md)
- [ARCHITECTURE.md](../ARCHITECTURE.md)

## Hardware and drivers

- [DRIVER_SYSTEM.md](DRIVER_SYSTEM.md)
- [DRIVER_STRATEGY.md](DRIVER_STRATEGY.md)
- [DRIVER_DATABASE.md](DRIVER_DATABASE.md)
- [HARDWARE_SUPPORT.md](HARDWARE_SUPPORT.md)
- [LINUX_DRIVER_COVERAGE.md](LINUX_DRIVER_COVERAGE.md)

## Native platform and security

- [NATIVE_FORMATS.md](NATIVE_FORMATS.md)
- [AWE_CAPSULE.md](AWE_CAPSULE.md)
- [AWE_XENOSENSE.md](AWE_XENOSENSE.md)
- [ECOSYSTEM.md](ECOSYSTEM.md)

## Engineering principle

A document can describe a target, but it cannot turn a target into an implementation. A roadmap checkbox becomes complete only after the corresponding code builds, tests, and—when hardware behavior is involved—boots or exercises successfully under the declared validation environment.

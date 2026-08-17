# AWE_OS Documentation Index

This directory is the engineering documentation surface for AWE_OS. Code is authoritative for implemented behavior; these documents describe contracts, design, validation and roadmap.

## Start here

1. [ENGINEERING_GUIDE.md](ENGINEERING_GUIDE.md) — how the repository is structured and how changes are validated.
2. [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — implementation status snapshot.
3. [PRODUCT_READINESS.md](../PRODUCT_READINESS.md) — product gates and evidence standard.
4. [AWE_OS_100_PERCENT_MASTER_PLAN.md](AWE_OS_100_PERCENT_MASTER_PLAN.md) — complete roadmap to 1.0.

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

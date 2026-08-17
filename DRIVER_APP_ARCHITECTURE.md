# AWE_OS Driver + Application Separation

## Decision

AWE_OS is now designed as a **microkernel-first system**:

- `CellKernel` = minimal trusted computing base.
- `driverd` = isolated, high-speed driver microkernel/service plane.
- `appd` = isolated application/package service plane.
- Native AWE drivers and applications are the primary ecosystem.
- Linux/Windows/Android support is implemented as compatibility services, never as CellKernel dependencies.

## What was found

The repository previously exported a large `kernel::drivers` module containing
native/compatibility abstractions, VirtIO, Linux lifecycle/recovery machinery,
Windows and Android adapters, registries, installers and resource/health
management. The driver tree is therefore substantially larger than a normal
minimal kernel driver boundary. fileciteturn49file0

`kernel/src/lib.rs` previously exported `pub mod drivers;`; that export has now
been removed. fileciteturn50file0

The old driver source remains in the repository as migration material, but it
is not part of CellKernel's public module graph anymore. fileciteturn51file0

## Runtime topology

```text
                         +------------------+
                         |    AWE Loader     |
                         +---------+--------+
                                   |
                         +---------v--------+
                         |    CellKernel    |
                         |------------------|
                         | MM / scheduler   |
                         | IPC / capabilities|
                         | process isolation |
                         +----+---------+---+
                              |         |
                    capability IPC      | capability IPC
                              |         |
                    +---------v--+   +--v---------+
                    |  driverd   |   |    appd    |
                    |------------|   |-------------|
                    | AWE native |   | AWE apps    |
                    | VirtIO     |   | packages    |
                    | Linux      |   | sandbox     |
                    | Windows    |   | lifecycle   |
                    | Android    |   | services    |
                    +------------+   +-------------+
```

## Driver classes

1. AWE native hardware drivers.
2. VirtIO drivers for QEMU/virtualized devices.
3. Linux compatibility drivers/adapters.
4. Windows compatibility adapters.
5. Android/vendor compatibility adapters.
6. Bus/resource/health/supervision services.

All six classes execute outside CellKernel.

## Application classes

- system shell and terminal;
- files/storage UI;
- settings/control center;
- task/process monitor;
- network manager;
- package/store manager;
- developer toolchain;
- compatibility launchers.

## Performance target

The driver boundary is intentionally small and designed for:

- fixed-size control messages;
- shared-memory/ring-buffer data paths;
- zero-copy transfers where safe;
- capability handles for MMIO/DMA/IRQ;
- per-driver watchdogs;
- fault quarantine and restart;
- no driver code in the kernel hot path.

This makes the **trusted kernel smaller** while allowing the driver plane to
be much larger without turning every driver bug into a kernel failure.

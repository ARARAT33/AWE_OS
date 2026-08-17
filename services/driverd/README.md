# AWE Driver Microkernel (`driverd`)

`driverd` is the **hardware-services plane** of AWE_OS. It is intentionally separate from `CellKernel`.

## Kernel boundary

CellKernel owns only:

- memory and address-space primitives;
- scheduling and IPC primitives;
- capability/security enforcement;
- interrupt delivery boundary;
- the minimal driver-service endpoint.

`driverd` owns:

- hardware discovery and binding;
- native AWE drivers;
- VirtIO drivers;
- Linux driver compatibility;
- Windows driver compatibility adapters;
- Android driver compatibility adapters;
- driver install/package logic;
- driver health, recovery and quarantine;
- DMA/MMIO resource ownership after capability grant.

## Isolation model

```text
AWE Loader
    |
    v
CellKernel  <---- capability/IPC ---->  driverd
    |                                      |
    |                              +-------+--------+
    |                              |                |
    |                         AWE Native        Compatibility
    |                         Drivers           Drivers
    |                              |          Linux/Windows/Android
    |                              +-------+--------+
    |                                      |
    +-------------------- devices ---------+
```

A driver fault must be recoverable by restarting or quarantining the driver service without requiring a kernel restart.

## Performance rules

1. No allocation in the hot driver dispatch path unless explicitly required.
2. Fixed-capacity registries during bootstrap.
3. Capability handles instead of global kernel pointers.
4. Shared-memory/ring-buffer IPC for high-throughput data paths.
5. Interrupts are delivered through the kernel boundary; device work runs in driverd.
6. Compatibility layers are never linked into CellKernel.

## Migration status

The historical driver implementation tree currently lives under `kernel/src/drivers/` but is **no longer exported or compiled by CellKernel**. It is a migration source/archive for the standalone driver plane. Drivers are migrated into `driverd` in small, testable groups rather than copied blindly.

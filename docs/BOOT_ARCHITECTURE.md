# AWEOS Universal Boot Architecture

AWEOS does not use one magical boot binary for every machine. It uses a common boot protocol with small architecture-specific entry adapters.

## Target families

| Family | 32-bit | 64-bit | Initial target |
|---|---:|---:|---|
| x86 | yes | yes | BIOS/UEFI PCs |
| ARM | yes | yes | embedded/mobile/boards |
| RISC-V | yes | yes | boards/SoCs |

## Common boot contract

The boot stage provides:

- kernel image and module locations
- memory map
- CPU topology
- firmware/device-tree information
- framebuffer when available
- random seed when firmware supplies one
- boot command line
- verified-image metadata

The kernel then performs architecture-specific CPU, MMU and interrupt setup.

## Boot stages

```text
Firmware
  -> AWE Boot Adapter
  -> image verification
  -> memory map / hardware description
  -> architecture entry
  -> CellKernel early init
  -> SMP / MMU / interrupts
  -> init service
```

## Design requirement

The common protocol is intentionally tiny. Hardware-specific code belongs in adapters, not in the portable kernel. This permits the same higher layers to run on desktops, servers, phones and embedded boards.

## Validation

A boot target is considered supported only after successful boot and automated smoke tests on a real target or a reproducible emulator. 32/64-bit support means that the corresponding toolchain, ABI, boot entry, pointer width, page-table format and scheduler context are implemented and tested; it is not merely a label in a configuration file.

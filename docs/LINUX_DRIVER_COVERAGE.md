# AWE_OS Linux Driver Coverage Plan

AWE_OS targets broad real-hardware coverage by using Linux as a **hardware compatibility reference**, not by blindly copying the Linux kernel into AWE_OS.

Linux's upstream tree exposes a very large driver surface under `drivers/`, including PCI, USB, MMC, TTY, PHY, MTD, DMA, GPIO, I3C, I2C, TEE, sound and many other subsystems. The upstream tree and its build metadata are the inventory source for this plan.

## Driver migration rule

Every driver entering AWE_OS must be one of:

1. a clean-room/native Rust AWE driver implementing the hardware specification;
2. a separately reviewed port whose source, license and copyright notices are preserved;
3. an explicitly permitted third-party implementation with compatible licensing.

A Linux driver is **not** copied merely because it exists upstream. Linux GPL-licensed code remains under its applicable license and is not silently relicensed as AWE_OS code.

## Coverage matrix

| Subsystem | Linux upstream inventory | AWE_OS target | Status |
|---|---|---|---|
| PCI/PCIe | `drivers/pci` | native Rust bus + driver discovery | foundation |
| USB/xHCI | `drivers/usb` | native host controller + device model | planned |
| DMA | `drivers/dma` | IOMMU-aware DMA HAL | planned |
| GPIO | `drivers/gpio` | capability-scoped GPIO | planned |
| I2C | `drivers/i2c` | native bus HAL | planned |
| I3C | `drivers/i3c` | native bus HAL | planned |
| MMC/SD | `drivers/mmc` | storage driver family | planned |
| NVMe | Linux storage drivers | native NVMe | planned |
| AHCI/SATA | Linux storage drivers | native AHCI | planned |
| VirtIO | Linux virtualization drivers | native VirtIO transport/device family | foundation |
| TTY | `drivers/tty` | console/terminal | planned |
| PHY | `drivers/phy` | hardware PHY HAL | planned |
| MTD | `drivers/mtd` | flash abstraction | planned |
| Sound | `sound/drivers` and sound subsystem | native audio HAL | planned |
| TEE | `drivers/tee` | secure-world interface | planned |
| Network | Linux network drivers | native NIC HAL + VirtIO net first | planned |
| GPU/display | Linux DRM/GPU ecosystem | native display HAL | planned |

## Security requirements for every port

- No direct arbitrary DMA; DMA must be described by a device contract.
- MMIO regions must be validated and bounded.
- Interrupt ownership must be explicit.
- Device identity must be checked before binding.
- Driver failure must not corrupt unrelated kernel state.
- Privileged operations must pass AWE capability policy.
- Fuzz and negative tests are required for parser/protocol boundaries.
- Driver provenance and license metadata must be retained.

## Priority order

### Tier 0 — virtualization and boot
VirtIO PCI, VirtIO block, VirtIO network, VirtIO console, serial, PCI enumeration, ACPI.

### Tier 1 — common PC hardware
NVMe, AHCI, xHCI, USB HID, PS/2 fallback, Intel/AMD NIC families, framebuffer/GOP.

### Tier 2 — broader desktop/server
I2C/I3C, GPIO, SPI, audio, GPU/display, Wi-Fi, Bluetooth, sensors.

### Tier 3 — embedded/mobile and specialist hardware
MMC, MTD, PHY families, platform buses, TEE, SoC-specific controllers.

The objective is not to claim "all Linux drivers" prematurely. The objective is to systematically convert the upstream hardware inventory into tested, isolated AWE drivers until the coverage matrix is complete.

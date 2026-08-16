# AWE_OS Universal Driver Database

AWE_OS targets broad hardware compatibility through a **driver knowledge and verification database**, not by executing foreign kernel driver ABIs.

## Sources

The database can index hardware/driver metadata from:

- Linux
- Android/Linux vendor trees
- Windows driver metadata and documented hardware IDs
- BSD families
- UEFI/ACPI/PCI specifications
- VirtIO/QEMU/KVM
- Vendor-published programming manuals

Source code is imported only when its license and provenance permit redistribution and after an explicit port/review step.

## Offline database

The installed system keeps a compact offline index containing:

- PCI vendor/device/class IDs
- USB VID/PID/class IDs
- ACPI/HID/compatible strings
- driver family and version
- source/provenance
- AWE driver ABI version
- security verification state
- required MMIO/DMA/IRQ capabilities
- supported hardware revisions

The offline index must be sufficient to identify hardware and select a previously verified driver without network access.

## Online database

The online service is an update source, never an authority that can silently execute code. Updates are:

1. downloaded into an isolated staging area;
2. signature/hash checked;
3. provenance and license metadata checked;
4. compatibility manifest validated;
5. driver package verified;
6. only then made available to the driver manager.

A failed update leaves the previous known-good database untouched.

## Universal compatibility model

AWE_OS uses a **Cross-OS Driver Compatibility Layer**. A Linux, Android, Windows or BSD driver is never loaded directly merely because its hardware ID matches. Instead, the compatibility database maps the hardware to a reviewed AWE adapter or native AWE driver.

This prevents a foreign kernel ABI from becoming part of the AWE trusted computing base.

## Driver quality states

`Unknown -> Indexed -> Ported -> UnitTested -> Emulated -> HardwareTested -> Signed -> Stable`

Only `Signed` and policy-approved `Stable` drivers may be enabled automatically in a production profile.

## Goal

The long-term goal is maximum practical hardware coverage across PCs, servers, laptops, ARM devices, embedded boards and virtual machines. **100% flawless compatibility cannot honestly be promised in advance** because hardware revisions, undocumented devices, firmware bugs and vendor-specific behavior exist. AWE_OS instead uses explicit verification states and safe fallback behavior so unsupported hardware fails closed rather than corrupting the kernel.

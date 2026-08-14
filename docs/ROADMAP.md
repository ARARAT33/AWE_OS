# AWEOS Engineering Roadmap

## Phase 0 — foundation
- [x] `no_std` Rust kernel entry
- [x] versioned BootInfo ABI
- [x] architecture module boundaries
- [x] capability primitives
- [x] syscall ABI draft
- [x] memory-region primitives

## Phase 1 — bootable kernel
- [ ] reproducible BIOS image
- [ ] UEFI application loader
- [ ] ELF/AWE image loading
- [ ] signed image verification
- [ ] x86 protected/long mode handoff
- [ ] x86_64 GDT/IDT
- [ ] AArch64 exception vectors
- [ ] ARM exception vectors
- [ ] RISC-V trap vectors

## Phase 2 — memory and CPU
- [ ] physical frame allocator
- [ ] x86_64 4-level/5-level paging
- [ ] ARM MMU/page tables
- [ ] RISC-V Sv32/Sv39/Sv48 support
- [ ] kernel heap
- [ ] timers
- [ ] APIC/GIC/CLINT or platform timer
- [ ] SMP bring-up
- [ ] context switching
- [ ] preemptive scheduler

## Phase 3 — kernel services
- [ ] process/thread objects
- [ ] syscall dispatcher
- [ ] capability table
- [ ] IPC channels
- [ ] shared-memory transport
- [ ] VFS/AWEFS
- [ ] init/service manager
- [ ] logging and crash diagnostics

## Phase 4 — hardware
- [ ] PCI/PCIe
- [ ] USB
- [ ] NVMe
- [ ] AHCI/SATA
- [ ] eMMC/SD
- [ ] HID
- [ ] Ethernet
- [ ] Wi-Fi controller framework
- [ ] framebuffer
- [ ] GPU abstraction
- [ ] audio

## Phase 5 — applications
- [ ] AWOSA executable/package format
- [ ] native SDK
- [ ] terminal
- [ ] shell
- [ ] file manager
- [ ] display server/compositor
- [ ] AYUI
- [ ] package manager

## Phase 6 — compatibility
- [ ] POSIX/Linux syscall compatibility
- [ ] Linux ELF runtime
- [ ] Windows API/runtime compatibility
- [ ] Android runtime/ABI compatibility

Compatibility is implemented progressively and tested per API family. “Supports all applications/drivers” is not considered complete until representative software and hardware test matrices pass.

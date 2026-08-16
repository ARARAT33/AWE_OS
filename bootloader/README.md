# AWE Bootloader Contract

The AWE bootloader is the only first-stage loader supported by the AWE_OS release contract.

## Security goals

1. Accept only the AWE boot image format and matching architecture.
2. Validate image length, alignment and entry-point bounds before execution.
3. Validate loader-to-kernel ABI version and magic.
4. Build and validate a complete memory-map handoff.
5. Refuse malformed or ambiguous memory regions.
6. Provide deterministic failure codes over the debug console.
7. Never execute an unverified kernel payload.
8. Reserve loader-owned memory until the kernel explicitly accepts the handoff.
9. Prepare a measured-boot extension point for future TPM/UEFI Secure Boot integration.
10. Keep the trusted boot path small and auditable.

## Planned production path

UEFI -> AWE image validation -> optional signature/measurement -> architecture/platform checks -> memory map normalization -> framebuffer/ACPI handoff -> kernel entry -> kernel acknowledges handoff -> loader-owned resources released.

The bootloader is not considered 100% complete until an automated QEMU UEFI boot test reaches the kernel's deterministic `Running` state and malformed-image tests demonstrate fail-closed behavior.

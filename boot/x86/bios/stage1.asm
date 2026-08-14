; AWEOS BIOS boot sector skeleton.
; 16-bit real mode: establish a known CPU state and chain to a loader stage.
; The final image builder must place the stage-2 sector count/address here.
BITS 16
ORG 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti

    ; Preserve BIOS boot drive in DL for stage-2.
    mov [boot_drive], dl

    ; Stage-2 disk loading is intentionally kept out of this 446-byte MBR
    ; area. The image builder supplies a contiguous stage-2 region.
    jmp 0x0000:0x7E00

boot_drive db 0

times 510-($-$$) db 0
dw 0xAA55

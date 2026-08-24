/* AWEOS 16-bit BIOS Stage1 / El Torito / MBR Boot Sector */
.code16
.text
.global _start
_start:
    jmp $0x0000, $real_start

real_start:
    cli
    xor %ax, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov $0x7C00, %sp
    sti

    /* Save BIOS boot drive number passed in DL */
    mov %dl, boot_drive

    /* Print boot message */
    mov $msg_boot, %si
1:
    lodsb
    orb %al, %al
    jz 2f
    mov $0x0E, %ah
    mov $0x07, %bx
    int $0x10
    jmp 1b
2:

    /* Load kernel using Extended Read (INT 13h, AH=0x42) */
    mov $dap, %si
    mov boot_drive, %dl
    mov $0x42, %ah
    int $0x13
    jnc 3f

    /* If AH=0x42 fails, print 'E' and halt */
    mov $0x0E45, %ax
    int $0x10
halt_loop:
    cli
    hlt
    jmp halt_loop

3:
    /* Enable A20 line via Fast A20 port 0x92 */
    in $0x92, %al
    or $0x02, %al
    out %al, $0x92

    /* Load 32-bit GDT */
    lgdt gdt_descriptor

    /* Enable Protected Mode in CR0 */
    mov %cr0, %eax
    or $0x00000001, %eax
    mov %eax, %cr0

    /* Far jump to 32-bit code segment */
    ljmp $0x08, $pm_start

.code32
pm_start:
    mov $0x10, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss
    mov $0x00090000, %esp

    /* Copy kernel sectors from buffer at 0x10000 to 0x100000 (1 MiB) */
    mov $0x00010000, %esi
    mov $0x00100000, %edi
    movzx (dap_sectors), %ecx
    shl $9, %ecx           /* ecx = dap_sectors * 512 bytes */
    shr $2, %ecx           /* ecx = dword count */
    rep movsl

    /* Enter kernel _start at 0x00101000 with Multiboot2 magic in EAX */
    mov $0x36D76289, %eax
    xor %ebx, %ebx
    mov $0x00101000, %ecx
    jmp *%ecx

.align 4
boot_drive: .byte 0
msg_boot: .asciz "Booting AWEOS...\r\n"

.align 4
.global dap
dap:
    .byte 16        /* DAP size */
    .byte 0         /* Reserved */
.global dap_sectors
dap_sectors:
    .word 512       /* Sector count (512 * 512 = 256 KB) */
.global dap_buffer
dap_buffer:
    .word 0x0000    /* Buffer offset */
    .word 0x1000    /* Buffer segment (0x1000:0x0000 = 0x10000) */
.global dap_lba
dap_lba:
    .quad 16        /* Starting LBA sector */

.align 8
gdt_start:
    .quad 0x0000000000000000               /* Null descriptor */
    .quad 0x00CF9A000000FFFF               /* 32-bit Code: Base 0, Limit 4GB */
    .quad 0x00CF92000000FFFF               /* 32-bit Data: Base 0, Limit 4GB */
gdt_end:

gdt_descriptor:
    .word gdt_end - gdt_start - 1
    .long gdt_start

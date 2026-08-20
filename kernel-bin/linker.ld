ENTRY(_start)

SECTIONS
{
    . = 1M;

    .note.Xen : ALIGN(4)
    {
        KEEP(*(.note.Xen))
    }

    .multiboot_header : ALIGN(8)
    {
        KEEP(*(.multiboot_header))
        KEEP(*(.multiboot2_header))
    }

    .text : ALIGN(4K)
    {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /DISCARD/ :
    {
        *(.eh_frame*)
        *(.comment)
        *(.note*)
    }
}

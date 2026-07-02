ENTRY(_start)

SECTIONS {
    /* we are loading the kernel starting from 1MB */
    . = 1M;

    .boot :
    {
        /* the multiboot header must be located at the beginning */
        KEEP(*(.multiboot))
    }

    .text :
    {
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss .bss.*)
    }
}
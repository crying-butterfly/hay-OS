global _start

extern kernel_main


; --- Multiboot 1 Header ---

section .multiboot

align 4

dd 0x1BADB002 ; Magic Number

dd 0x00 ; Flags

dd -(0x1BADB002 + 0x00) ; Checksumm


; --- BSS Section

section .bss

align 4096 ; Page Tables must be 4kb

p4_table: resb 4096

p3_table: resb 4096

p2_table: resb 4096

stack_bottom: resb 16384

stack_top:


; --- 32-Bit Code: preperation for 64 bit

section .text

bits 32

_start:

mov esp, stack_top


; 1. Page link page tables (P4 -> P3 -> P2)

mov eax, p3_table

or eax, 0b111 ; Set 'Present' and 'Writable' Flags

mov [p4_table], eax


mov eax, p2_table

or eax, 0b111

mov [p3_table], eax


; 2. Configure P2 to map a 2-megabyte huge page to address 0.

mov eax, 0x0

or eax, 0b10000011 ; Present + Writable + Huge Page (Bit 7)

mov [p2_table], eax


;3. Provide the address of the P4 table to the processor (via the CR3 register)

mov eax, p4_table

mov cr3, eax


; 4. activate physical adress extension (PAE)

mov eax, cr4

or eax, 1 << 5

mov cr4, eax


; 5. activate the 64bit long modes MSRs

mov ecx, 0xC0000080

rdmsr

or eax, 1 << 8

wrmsr


; 6. activate paging so we are fully in the capability mode

mov eax, cr0

or eax, 1 << 31

mov cr0, eax


;7. load the 64 bit gdt and with the far jump we can finally jump to the 64 bit mode

lgdt [gdt64.pointer]

jmp gdt64.code_segment:long_mode_start


hlt


; --- 64-Bit Code

bits 64

long_mode_start:

; Zero out all data segment registers (no longer required/allowed in 64-bit mode) e

mov ax, 0
mov ss, ax
mov ds, ax
mov es, ax
mov fs, ax
mov gs, ax


; jumping to the c kernel

call kernel_main


.hang:

hlt

jmp .hang


; 64-Bit Global Descriptor Table (GDT) 

section .rodata

gdt64:

dq 0 ; Null entry

.code_segment: equ $ - gdt64

;set code,app present and 64 bit flags 

dq (1<<43) | (1<<44) | (1<<47) | (1<<53)

.pointer:

dw $ - gdt64 - 1

dq gdt64 
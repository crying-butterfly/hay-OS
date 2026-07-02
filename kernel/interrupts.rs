use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{println}; // if needed add print
use crate::vga::WRITER;
use core::fmt::Write;
use x86_64::instructions::port::Port;
use crate::pic::PICS;
use crate::gdt;
use x86_64::VirtAddr;
use core::arch::naked_asm;

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
pub const KEYBOARD_INTERRUPT_INDEX: u8 = 33;
pub const TIMER_INTERRUPT_INDEX: u8 = crate::pic::PIC_1_OFFSET;
pub const PRIMARY_ATA_INTERRUPT_INDEX: u8 = crate::pic::PIC_1_OFFSET + 14;
pub const SECONDARY_ATA_INTERRUPT_INDEX: u8 = crate::pic::PIC_1_OFFSET + 15;

pub fn idt_init() {
    unsafe {
        IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        IDT.divide_error.set_handler_fn(divide_error_handler);
        IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.general_protection_fault.set_handler_fn(gpf_handler);
        
        let timer_addr = timer_interrupt_handler as *const () as u64;

        unsafe {
            IDT[TIMER_INTERRUPT_INDEX]
                .set_handler_addr(VirtAddr::new(timer_addr))
                .set_present(true)
                .set_code_selector(crate::gdt::GDT.1.kernel_code_selector);
        }
        
        IDT[KEYBOARD_INTERRUPT_INDEX].set_handler_fn(keyboard_interrupt_handler);
        IDT[PRIMARY_ATA_INTERRUPT_INDEX].set_handler_fn(primary_ata_interrupt_handler);
        IDT[SECONDARY_ATA_INTERRUPT_INDEX].set_handler_fn(secondary_ata_interrupt_handler);
        IDT.load();
    }
}

// kernel fault
fn kernel_fault(fault_name: &str, stack_frame: &InterruptStackFrame, error_code: Option<u64>) -> ! {
    let mut writer = WRITER.lock();
    writer.color_code = 0x1F; 
    writer.clear_screen();

    writer.write_string("=======================================================================\n");
    writer.write_string("                       HARDWARE EXCEPTION: KERNEL FAULT                \n");
    writer.write_string("=======================================================================\n\n");

    // read the rip adress
    let rip = stack_frame.instruction_pointer.as_u64();
    
    let _ = write!(writer, "FAULT TYPE: {}\n", fault_name);
    let _ = write!(writer, "CRASHED AT RIP: {:#018x}\n\n", rip);
    
    if let Some(err) = error_code {
        let _ = write!(writer, "ERROR CODE: {:#x}\n\n", err);
    }

    let _ = write!(writer, "STACK FRAME DUMP:\n{:#?}\n", stack_frame);

    loop {}
}

#[unsafe(naked)]
pub unsafe extern "C" fn timer_interrupt_handler() {
    naked_asm!(
        // Secure the full cpu state of the current task on his own stack
        "push rbp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push rax",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",

        // pass the current stackpointer as first argument to the register RDI
        "mov rdi, rsp",
        
        // alings the stack to 16 byte
        "and rsp, -16",
        
        "call handle_timer_and_schedule",
        
        // we overwrite the RSP so the stack switch is complete
        "mov rsp, rax",

        // restore the secured CPU State of the new task of his stack
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",

        // iretq jumps back restores the RSP,PIC,CS and restores the RFLAGS of the new tasks
        "iretq",
    );
}

// the handlers

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let rip = stack_frame.instruction_pointer.as_u64();
    println!("EXCEPTION: BREAKPOINT at RIP: {:#018x}", rip);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, error_code: u64) -> !
{
    kernel_fault("DOUBLE FAULT", &stack_frame, Some(error_code));
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    kernel_fault("DIVIDE BY ZERO", &stack_frame, None);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kernel_fault("INVALID OPCODE", &stack_frame, None);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode)
{
    kernel_fault("PAGE FAULT", &stack_frame, Some(error_code.bits()));
}

extern "x86-interrupt" fn gpf_handler(
    stack_frame: InterruptStackFrame, error_code: u64)
{
    kernel_fault("GENERAL PROTECTION FAULT", &stack_frame, Some(error_code));
}

extern "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;

    static KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::MapLettersToUnicode)
    );
   
    let mut keyboard = KEYBOARD.lock();
    // Port 0x60 is the data port of the keyboard controller
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    // handle scancodes
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    // passing the letter to the shell module
                    crate::shell::handle_input(character);
                }
                DecodedKey::RawKey(key) => {
                    stub!("todo: add special keys");
                }
            }
        }
    }

    // signalize the pic that the interrupt has been processesd
    unsafe {
        PICS.lock().notify_end_of_interrupt(KEYBOARD_INTERRUPT_INDEX);
    }
}

#[no_mangle]
pub extern "C" fn handle_timer_and_schedule(current_rsp: u64) -> u64 {
    // signalizes the pic that the interrupt got processed so the hard ware lock will be ended
    unsafe {
        PICS.lock().notify_end_of_interrupt(TIMER_INTERRUPT_INDEX);
    }

    if let Some(mut scheduler) = crate::task::SCHEDULER.try_lock() {
        return scheduler.pick_next_task(current_rsp);
    }

    // if the scheduler is locked stay in current task (Fallback)
    current_rsp
}

extern "x86-interrupt" fn primary_ata_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(PRIMARY_ATA_INTERRUPT_INDEX);
    }
}

extern "x86-interrupt" fn secondary_ata_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(SECONDARY_ATA_INTERRUPT_INDEX);
    }
}
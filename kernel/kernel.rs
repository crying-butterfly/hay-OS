#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use x86_64::instructions;
use core::panic::PanicInfo;
use crate::vga::WRITER;
use crate::vga::clear_screen;
use x86_64::VirtAddr;
extern crate alloc;

#[macro_use]
pub mod macros;
pub mod vga;
pub mod interrupts;
pub mod pic;
pub mod shell;
pub mod gdt;
pub mod task;
pub mod allocator;
pub mod memory;


#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    clear_screen();
    
    gdt::init();
    interrupts::idt_init();
    pic::init();
    allocator::init();

    crate::println!("Welcome to Hay OS\n");

    let phys_mem_offset = VirtAddr::new(0x0);
    let mut mapper = unsafe { memory::init_mapper(phys_mem_offset) };
    // multitasking setup
    {
        let mut scheduler = task::SCHEDULER.lock();

        // the scheduler will get overwriten and safed at the first switch
        scheduler.tasks[0] = Some(task::Task {
            id: 0,
            stack_pointer: 0,
            _stack_mem: alloc::vec![],
        });

        scheduler.tasks[1] = Some(task::Task::new_user(1, shell::shell_task_main, &mut mapper));

        scheduler.current_task = 0;
    }
    
    // here the timer and the shell starts
    instructions::interrupts::enable();
    
    // the main thread stays in a protected llop
    loop {
        instructions::hlt();
    }
}

// kernel panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! { 
    let mut writer = WRITER.lock();
    writer.color_code = 0x4F; 

    writer.clear_screen();

    writer.write_string("=======================================================================\n");
    writer.write_string("                       KERNEL PANIC: SOFTWARE CRASH                    \n");
    writer.write_string("=======================================================================\n\n");

    use core::fmt::Write;
    let _ = write!(writer, "{}", info);

    loop {}
}
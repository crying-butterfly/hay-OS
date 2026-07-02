use pic8259::ChainedPics;
use spin::Mutex;

// the master PIC starts at interrupt vector 32 because the CPU is usin 0-31 and 32 is the first free slot
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // Interrupt Vector 40

pub static PICS: Mutex<ChainedPics> = unsafe {
    Mutex::new(ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET))
};

pub fn init() {
    unsafe {
        // initalizes both PICs and remaps the interrupt vectors
        PICS.lock().initialize();
    }
}
use x86_64::{
    structures::paging::{
        OffsetPageTable, PageTable, Mapper, Page, PageTableFlags, Translate, PhysFrame, 
    },
    VirtAddr,
};

use x86_64::structures::paging::mapper::TranslateResult;

pub unsafe fn init_mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // return mutable reference
}

pub fn make_user_accessible(
    mapper: &mut OffsetPageTable,
    start_addr: VirtAddr,
    size_in_bytes: u64,
) {
    use x86_64::structures::paging::Size2MiB;

    let start_page = Page::<Size2MiB>::containing_address(start_addr);
    let end_page = Page::<Size2MiB>::containing_address(start_addr + size_in_bytes - 1u64);

    for page in Page::range_inclusive(start_page, end_page) {
        // use the translate trait to determine the curennt flags of the page
        if let TranslateResult::Mapped { flags, .. } = mapper.translate(page.start_address()) {
            let mut new_flags = flags;
            // adds the user accesible flag
            new_flags.insert(PageTableFlags::USER_ACCESSIBLE);

            // refresh the flags
            if let Ok(flush) = unsafe { mapper.update_flags(page, new_flags) } {
                flush.flush(); // empty TLB cache for this page
            }
        }
    }
}


use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PhysFrame, Size4KiB, Translate};
use x86_64::{PhysAddr, VirtAddr};


unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub unsafe fn init_mapper(phys_offset: u64) -> OffsetPageTable<'static> {
    let phys_offset: VirtAddr = VirtAddr::new(phys_offset);
    let level_4_table = active_level_4_table(phys_offset);
    OffsetPageTable::new(level_4_table, phys_offset)
}

pub fn translate_addr(mapper: &OffsetPageTable, address: VirtAddr) -> Option<PhysAddr> {
    mapper.translate_addr(address)
}
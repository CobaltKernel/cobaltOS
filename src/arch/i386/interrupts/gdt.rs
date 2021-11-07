//! Handles Interacting With the GDT & Setting Segment Selectors

use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use lazy_static::lazy_static;
use x86_64::structures::gdt::SegmentSelector;


#[allow(deprecated)]
use x86_64::instructions::segmentation::set_cs;
use x86_64::instructions::tables::load_tss;

/// The IST Index For The Double Fault Handler Stack
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
	#[allow(missing_docs)]
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

	#[allow(missing_docs)]
	pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
		let mut gdt = GlobalDescriptorTable::new();
		let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
		let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
		let data = gdt.add_entry(Descriptor::kernel_data_segment());
		let user_code = gdt.add_entry(Descriptor::user_code_segment());
		let user_data = gdt.add_entry(Descriptor::user_data_segment());
		(gdt, Selectors {code_selector, tss_selector, data, user_code, user_data})
	};
}

#[allow(dead_code)]
/// Keeps Track Of The Current Segment Selectors
pub struct Selectors {
	/// Kernel Code Selector
	code_selector: SegmentSelector,
	/// TSS Selector
	tss_selector:  SegmentSelector,
	/// Kernel Data Selector
    data: SegmentSelector,

	/// Usermode Code Selector
    pub user_code: SegmentSelector,
	/// Usermode Data Selector
    pub user_data: SegmentSelector,
}

/// Setup & Load The GDT, Set the CS register & load the TSS
pub fn init() {
	GDT.0.load();
	unsafe {
		#[allow(deprecated)]
		set_cs(GDT.1.code_selector);
		load_tss(GDT.1.tss_selector);
	}
}

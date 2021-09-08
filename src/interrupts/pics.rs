use pic8259::ChainedPics;
use spin;
use lazy_static::lazy_static;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static PICS: spin::Mutex<ChainedPics> = 
	spin::Mutex::new(unsafe {ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});

pub unsafe fn init() {
	PICS.lock().initialize();
} 

pub fn send_eoi(index: u8) {
	unsafe {
		PICS.lock().notify_end_of_interrupt(index);
	}
}

use pic8259::ChainedPics;
use spin;
use lazy_static::lazy_static;
use x86_64::instructions::port::Port;

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

pub fn is_spurious(irq: u8) -> bool {
	return (get_isr() & (1 as u16) << irq) == 0;
}

pub fn get_isr() -> u16 {
	let mut master_cmd: Port<u8> = Port::new(0x20);
	let mut master_dat: Port<u8> = Port::new(0x21);
	let mut slave_cmd: Port<u8> = Port::new(0xA0);
	let mut slave_dat: Port<u8> = Port::new(0xA1);
	let mut high: u16 = 0;
	let mut low: u16 = 0;
	unsafe {
		master_cmd.write(0x0B);
		slave_cmd.write(0x0B);

		high = (slave_dat.read() as u16) << 8;
		low = (master_dat.read() as u16) << 0;
	}

	return high | low;



}

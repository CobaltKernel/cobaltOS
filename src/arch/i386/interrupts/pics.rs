//! Handles Interacting With The 8259 PICS

use pic8259::ChainedPics;
use spin;
use x86_64::instructions::port::Port;

/// Remap PIC 1 To 32
pub const PIC_1_OFFSET: u8 = 32;
/// Remap PIC 2 To 40
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static PICS: spin::Mutex<ChainedPics> = 
	spin::Mutex::new(unsafe {ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});

/// Initialise The PICS
pub unsafe fn init() {
	PICS.lock().initialize();
} 

/// Notifiy The PICS of an End Of Interrupt
pub fn send_eoi(index: u8) {
	unsafe {
		PICS.lock().notify_end_of_interrupt(index);
	}
}

/// Check Whether Or Not An IRQ Is Spurious (ie Should Be Handled or not)
pub fn is_spurious(irq: u8) -> bool {
	let irq = irq - 32;
	//println!("Checking Interrupt #{}", irq);
	return (get_isr() & (1 as u16) << irq) == 0;
}


/// Get The Interrupt Status Register
pub fn get_isr() -> u16 {
	let mut master_cmd: Port<u8> = Port::new(0x20);
	let mut master_dat: Port<u8> = Port::new(0x21);
	let mut slave_cmd: Port<u8> = Port::new(0xA0);
	let mut slave_dat: Port<u8> = Port::new(0xA1);
	let high: u16;
	let low: u16;
	unsafe {
		master_cmd.write(0x0B);
		slave_cmd.write(0x0B);

		high = ((slave_dat.read() as u16) & 0xFF) << 8;
		low = master_dat.read() as u16;
	}

	return (high & 0xFF00) | (low & 0x00FF);
}

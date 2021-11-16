use x86_64::instructions::{port::*, interrupts::*};

use crate::log;

const MASTER_RATE: f64 = 1193810.0;
const COMMAND_PORT: u16 = 0x43;
const CH0_DATA: u16 = 0x40;
const SET_CH0_FREQ_CMD: u8 = 0x36;

pub fn set_freq(hz: f64) {
	without_interrupts(|| {
	let divisor: usize = (MASTER_RATE / hz) as usize;
	let actual: f64 = MASTER_RATE as f64 / divisor as f64;
	let mut command_port: Port<u8> = Port::new(COMMAND_PORT);
	let mut data_port: Port<u8> = Port::new(CH0_DATA);
	
	unsafe {
		command_port.write(SET_CH0_FREQ_CMD);
		data_port.write((divisor & 0xFF) as u8);
		data_port.write((divisor >> 8) as u8);
	}

	//log!("Set PIT Freq to {} (reload: {}, {} Hz)", hz, divisor, actual);
	});
}

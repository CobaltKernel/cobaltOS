use crate::{serial_print, serial_println};

#[cfg(feature = "80_cols")]
pub const SCREEN_WIDTH: usize = 80;

#[cfg(not(feature = "80_cols"))]
pub const SCREEN_WIDTH: usize = 40;


pub const PROGRESS_BAR_LEN: usize = 20; 

pub const fn screen_width() -> usize {
	return SCREEN_WIDTH;
}

pub struct ProgressBar<'a> {
	name: &'a str,
	value: usize,
	max: usize,
}

impl<'a> ProgressBar<'a> {
	pub fn new(value: usize, max: usize, name: &'a str) -> ProgressBar {
		ProgressBar {
			value,
			max,
			name,
		}
	}

	pub fn set_value(&mut self, value: usize) {
		self.value = value;
	} 

	pub fn draw(&self) {
		serial_print!("{} - [", self.name);
		let percentage: f64 = (self.value as f64 / self.max as f64);
		let blocks: usize = (percentage * PROGRESS_BAR_LEN as f64) as usize;

		for _ in 0..blocks {
			serial_print!("#");
		}

		for _ in blocks..PROGRESS_BAR_LEN {
			serial_print!(" ");
		}

		serial_print!("] - {:03.2}%    \r", percentage * 100.0);
		
	}
}

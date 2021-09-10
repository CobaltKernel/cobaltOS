use crate::{serial_print, serial_println};

#[cfg(feature = "80_cols")]
pub const SCREEN_WIDTH: usize = 80;

#[cfg(not(feature = "80_cols"))]
pub const SCREEN_WIDTH: usize = 40;

pub fn screen_width() -> usize {
	return SCREEN_WIDTH;
}

pub struct ProgressBar<'a> {
	name: &'a str,
	value: usize,
	scale: f64,
}

impl<'a> ProgressBar<'a> {
	pub fn new(value: usize, max: usize, name: &'a str) -> ProgressBar {
		ProgressBar {
			value,
			scale: (screen_width() as f64 / max as f64),
			name,
		}
	}

	pub fn set_value(&mut self, value: usize) {
		self.value = value;
	} 

	pub fn draw(&self) {
		serial_print!("{} - [", self.name);

		let fill: f64 = (self.value as f64) / (screen_width() as f64);

		for _ in 0..(((fill) * self.scale as f64) as usize) {
			serial_print!("#");
		}

		for _ in (((fill) * self.scale as f64) as usize)..screen_width() {
			serial_print!(" ");
		}
		
		serial_print!("] - Fill: {}, Scale: {}, Value: {}      \r", fill, self.scale, self.value);
	}
}
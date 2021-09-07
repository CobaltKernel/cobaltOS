use crate::systems::SystemBase;

#[cfg(feature = "vga")]
mod vga;
#[cfg(feature = "vga")]
pub use vga::VgaDriver as Driver;


#[cfg(not(feature = "vga"))]
pub struct DummyDriver;
#[cfg(not(feature = "vga"))]
pub use DummyDriver as Driver;


#[derive(Debug, Clone)]
pub struct GraphicsInfo {
	pub buffer_width: usize,
	pub buffer_height: usize,
	pub buffer_depth: usize,

	pub char_width: usize,
	pub char_height: usize,
	
	pub gl_compatible: bool,
	pub vk_compatible: bool,
}
pub struct Color(usize);
pub trait GraphicsDriver {
	fn set_pixel(&mut self, x: usize, y: usize, color: Color);
	fn swap(&mut self);
	fn draw_char(&mut self, x: usize, y: usize, chr: char);

	fn get_info(&self) -> GraphicsInfo;
} 


#[cfg(not(feature = "vga"))]
impl GraphicsDriver for DummyDriver {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
    }

    fn swap(&mut self) {
    }

    fn draw_char(&mut self, x: usize, y: usize, chr: char) {
    }

    fn get_info(&self) -> GraphicsInfo {
        GraphicsInfo { buffer_width: 80, buffer_height: 25, buffer_depth: 4, char_width: 8, char_height: 8, gl_compatible: false, vk_compatible: false }
    }
}



#[derive(Debug, Clone)]
pub struct GraphicsInfo {
	buffer_width: usize,
	buffer_height: usize,
	buffer_depth: usize,
	
	gl_compatible: bool,
	vk_compatible: bool,
}
pub struct Color(usize);
pub trait GraphicsDriver {
	fn set_pixel(&mut self, x: usize, y: usize, color: Color);
	fn swap(&mut self);

	fn get_info(&self) -> GraphicsInfo;
} 

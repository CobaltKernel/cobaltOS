use self::drivers::GraphicsDriver;

use super::SystemBase;

mod drivers;

use drivers::Driver;

pub struct Graphics;

impl Graphics {
    pub fn draw_str(x: usize, y: usize, text: &str) {
        let info = Driver.get_info();
        for (index, chr) in text.char_indices() {
            Driver.draw_char(x * info.char_width + index * info.char_width, y , chr);
        }
    }
}


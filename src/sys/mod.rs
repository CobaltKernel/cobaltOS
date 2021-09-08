pub mod timer;
pub mod pit;
pub mod ansi_widgets;

pub fn halt() -> ! {
	loop { timer::pause(0.1); }
}

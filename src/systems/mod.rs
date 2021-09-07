pub mod graphics;
pub mod output;

pub trait SystemBase {
	fn instance() -> &'static Self;
}


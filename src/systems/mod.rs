mod graphics;

pub trait SystemBase {
	fn instance() -> &'static Self;
}


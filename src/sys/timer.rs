use spin::Mutex;
use x86_64::instructions::hlt;
use super::pit::set_freq;
static TIMER: Mutex<u128> = Mutex::new(0);
pub const TICKS_PER_SECOND: f64 = 1000.6789606035205f64;
// TODO(Capt. Autism): Wrap contents into a without_interrupts closure to make interrupt-safe. 
pub fn increment() {
	*TIMER.lock() += 1;
}

// TODO(Capt. Autism): Wrap contents into a without_interrupts closure to make interrupt-safe.
pub fn clear() {
	*TIMER.lock() = 0;
}

pub fn init() {
	set_freq(TICKS_PER_SECOND);
}

pub fn pause(seconds: f64) {
	let ticks: usize = (seconds * TICKS_PER_SECOND) as usize;
	for _ in 0..=ticks {
		hlt();
	}
}



use std::fmt::Debug;
use std::fs;

use crate::emulator::Emulator;

mod emulator;
mod events;
mod sdl2_interaction;

pub trait LogError {
	fn elog(self, msg: &str);
}
impl<T, E: Debug> LogError for Result<T, E> {
	fn elog(self, msg: &str) {
		if let Err(e) = self {
			eprintln!("ERROR while {}: {:?}", msg, e);
		}
	}
}

pub trait LogWarning {
	fn wlog(self);
}
impl LogWarning for &str {
	fn wlog(self) {
		println!("WARNING: {}", self);
	}
}

pub trait LogInfo {
	fn log(self);
}
impl LogInfo for &str {
	fn log(self) {
		println!("INFO: {}", self);
	}
}

static FPS: f32 = 60.0;

fn main() {
	Emulator::new_chip8(
		FPS,
		12,
		fs::read(
			std::env::args()
				.nth(1).unwrap_or_else(|| "./roms/spinvaders.ch8".to_string())
			).unwrap()
	).run();
}
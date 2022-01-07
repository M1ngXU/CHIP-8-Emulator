use std::fs;
use crate::emulator::Emulator;

mod structs;
mod output;
mod event_manager;
mod app_state;
mod emulator;
mod logger;

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
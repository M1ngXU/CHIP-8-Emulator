use std::fmt::Debug;
use std::fs;
use sdl2::pixels::Color;

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

static SPEED_CHANGE_PER_KEYPRESS: f32 = 1.2;
static FPS: f32 = 60.0;
static PAUSE_TRANSPARENT_COLOR: Color = Color::RGBA(0xFF, 0xFF, 0xFF, 0x99);
static OPCODES_PER_FRAME: u32 = 12;
static SCREEN_WIDTH: u32 = 128;
static SCREEN_HEIGHT: u32 = 64;
static STARTING_SCALE: u32 = 10;
static STANDARD_BUZZ_FREQUENCY: f32 = 440.0;

fn main() {
	println!("---(SUPER) CHIP8 EMULATOR BY M1ngXU---");
	if let Some(path) = std::env::args().nth(1) {
		match fs::read(&path) {
			Ok(bin) => Emulator::new_chip8(FPS, OPCODES_PER_FRAME, bin).run(),
			Err(e) => eprintln!("Failed to read file \"{}\" - error: \"{}\".", path, e)
		}
	} else {
		eprintln!("No file specified as first parameter!");
	}
	println!("EMULATION TERMINATED");
}
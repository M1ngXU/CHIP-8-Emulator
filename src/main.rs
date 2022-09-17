#![feature(result_option_inspect)]

use rfd::FileDialog;
use sdl2::pixels::Color;
use std::fmt::Debug;
use std::fs;

use crate::emulator::{Emulator, End};

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

pub fn get_fd(sub_dir: &str) -> FileDialog {
    let mut fd = FileDialog::new();
    if let Ok(cur) = std::env::current_dir() {
        let joined = cur.join(sub_dir);
        if let Some(valid_path) = [joined, cur].into_iter().find(|p| p.exists()) {
            fd = fd.set_directory(valid_path);
        }
    }
    fd
}

fn main() {
    println!("---(SUPER) CHIP8 EMULATOR BY M1ngXU---");
    let mut emulator = Emulator::new_chip8(FPS, OPCODES_PER_FRAME);
    let mut arg_path = std::env::args().nth(1);
    'main: loop {
        if let Some(path) = arg_path.take().or_else(|| {
            get_fd("roms")
                .add_filter("Chip8 Binary", &["rom", "ch8", "bin"])
                .add_filter("all", &["*"])
                .set_title("Choose a Chip8 binary file")
                .pick_file()
                .and_then(|p| p.to_str().map(|s| s.to_owned()))
        }) {
            arg_path.take();
            match fs::read(&path) {
                Ok(bin) => {
                    while {
                        emulator.load_memory(bin.clone());
                        match emulator.run() {
                            End::Quit => break 'main,
                            End::Restart => true,
                            End::NewGame => continue 'main,
                        }
                    } {
                        println!("Restarting");
                    }
                }
                Err(e) => eprintln!("Failed to read file \"{}\" - error: \"{}\".", path, e),
            }
        } else {
            eprintln!("Failed to get file!");
        }
    }
    println!("EMULATION TERMINATED");
}

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::thread;
use std::time::SystemTime;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use crate::app_state::InputState;
use crate::interpreter::*;
use crate::logger::{LogInfo, LogWarning};
use crate::output::Output;

static FONT: [ u8; 80 ] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
/// what 'part' should be added/removed to the speed if f2/f3 are pressed? - the lower this number the greater the change
/// ```speed += old_speed / SPEEDCHANGE```
static SPEED_CHANGE: u16 = 5;

pub struct Emulator<T: Interpreter> {
    paused: Arc<AtomicBool>,
    fps: f32,
    opcodes_per_frame: u32,
    interpreter: T,
    speed: Arc<AtomicU16>,
    last_opcode: SystemTime,
    opcode_counter: u32
}
impl Emulator<Chip8Interpreter> {
    pub fn new_chip8(fps: f32, opcodes_per_frame: u32, opcodes: Vec<u8>) -> Self {
        let output = Arc::new(Mutex::new(Output::new(64, 32, 10)));
        let mut app_state = output.lock().unwrap().get_input_state();
        let mut interpreter = Chip8Interpreter::new(output.clone());
        interpreter.load_memory(FONT.to_vec(), 0);
        interpreter.load_memory(opcodes, 0x200);
        let speed = Arc::new(AtomicU16::new(1000));
        let paused = Arc::new(AtomicBool::new(false));

        let s = speed.clone();
        let p = paused.clone();
        thread::spawn(move || {
            let mut pause_before_focus_loss = false;
            let mut cheat_mode = false;
            let mut last_frame = SystemTime::now();
            let millis_between_frames = (1000.0 / fps) as u128;

            loop {
                app_state.update();
                if app_state.is_terminating() {
                    std::process::exit(0);
                }
                let mut output = output.lock().unwrap();
                let keyboard_state = app_state.get_keyboard_state();
                if last_frame.elapsed().unwrap().as_millis() > millis_between_frames {
                    output.update_screen();
                    last_frame = SystemTime::now();
                }
                if keyboard_state.just_pressed(Scancode::F1.into()) {
                    s.store(1000, Ordering::Relaxed);
                    "Resetted speed to 100%.".log();
                }
                if keyboard_state.just_pressed(Scancode::F2.into()) {
                    let mut new_speed = s.load(Ordering::Relaxed);
                    if new_speed >= 11 {
                        new_speed -= new_speed / SPEED_CHANGE;
                        s.store(new_speed, Ordering::Relaxed);
                        format!("Speed changed to {}%", new_speed / 10).as_str().log();
                    } else {
                        "Reached min speed!".wlog();
                    }
                }
                if keyboard_state.just_pressed(Scancode::F3.into()) {
                    let new_speed = s.load(Ordering::Relaxed);
                    let delta = new_speed / SPEED_CHANGE;
                    if new_speed <= u16::MAX - delta {
                        s.store(new_speed + delta, Ordering::Relaxed);
                        format!("Speed changed to {}%", new_speed / 10).as_str().log();
                    } else {
                        "Reached max speed!".wlog();
                    }
                }
                if keyboard_state.just_pressed(Scancode::F4.into()) {
                    cheat_mode = !cheat_mode;
                    if cheat_mode {
                        "CHEAT-MODE turned on!!! Drawing onto the screen modifies collisions ...".wlog();
                    } else {
                        "CHEAT-MODE turned off.".wlog();
                    }
                }
                if cheat_mode {
                    if let Some(&m) = app_state.get_mouse_button_state().currently_pressed_all().iter().next() {
                        let (x, y) = app_state.get_mouse_coordinates();
                        match m {
                            MouseButton::Left => {
                                output.set(x as u32, y as u32, true);
                                continue;
                            }, MouseButton::Right => {
                                output.set(x as u32, y as u32, false);
                                continue;
                            },
                            _ => {}
                        }
                    }
                }
                if keyboard_state.just_pressed(Scancode::F11.into()) || (
                    keyboard_state.just_pressed(Scancode::Return.into())
                        && keyboard_state.currently_pressed(Scancode::LAlt.into())
                ) {
                    output.toggle_fullscreen();
                    "Toggled fullscreen-mode.".log();
                }
                if keyboard_state.just_pressed(Scancode::Escape.into()) {
                    let cur_pause_state = p.load(Ordering::Relaxed);
                    p.store(!cur_pause_state, Ordering::Relaxed);
                    if !cur_pause_state {
                        "Paused emulation ...".log();
                    } else {
                        "Unpaused emulation ...".log();
                    }
                }
                if app_state.just_lost_focus() {
                    pause_before_focus_loss = p.load(Ordering::Relaxed);
                    p.store(true, Ordering::Relaxed);
                    output.stop_buzz();
                    "Lost focus.".log();
                }
                if app_state.just_gained_focus() {
                    p.store(pause_before_focus_loss, Ordering::Relaxed);
                    "Regained focus.".log();
                }
                if output.size_changed() || app_state.just_gained_focus() {
                    output.redraw_screen();
                }
            }
        });
        Self {
            paused,
            interpreter,
            fps,
            opcodes_per_frame,
            speed,
            last_opcode: SystemTime::now(),
            opcode_counter: 0
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn run(&mut self) {
        loop {
            if self.is_paused() {
                continue;
            }

            let wait_time = 1_000_000.0 / self.fps / self.opcodes_per_frame as f32 / self.speed.load(Ordering::Relaxed) as f32 * 1000.0;
            while wait_time > self.last_opcode.elapsed().unwrap().as_micros() as f32 {};
            self.last_opcode = SystemTime::now();

            self.interpreter.interpret_next();

            if self.opcode_counter % self.opcodes_per_frame == 0 {
                self.interpreter.next_frame();
            }
            self.opcode_counter += 1;
        }
    }
}
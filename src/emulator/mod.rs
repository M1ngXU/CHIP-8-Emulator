use std::collections::HashSet;
use std::sync::mpsc;
use std::time::SystemTime;

use fixed_bit_numbers::IntoEmpty;

use crate::emulator::interpreter::{Chip8Interpreter, Interpreter, InterpreterEvent};
use crate::events::app::AppEvent;
use crate::events::EventRedirectManager;
use crate::events::input::InputEvent;
use crate::LogError;
use crate::sdl2_interaction::audio_manager::AudioEvent;
use crate::sdl2_interaction::event_manager::{AppEventReceiver, AppEventSender, IncomingEvent};
use crate::sdl2_interaction::output::{Output, ScreenEvent};
use crate::sdl2_interaction::screen::Chip8ColorToBool;

pub mod interpreter;
mod fixed_bit_numbers;

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
/// with what should the speed be multiplied when increasing it?
pub static SPEED_CHANGE: f32 = 1.2;

pub struct Emulator<T: Interpreter> {
    fps: f32,
    opcodes_per_frame: u32,
    interpreter: T,
    last_opcode: SystemTime,
    opcode_counter: u32,
    interpreter_receiver: AppEventReceiver,
    app_state_event_sender: AppEventSender
}
impl Emulator<Chip8Interpreter> {
    pub fn new_chip8(fps: f32, opcodes_per_frame: u32, opcodes: Vec<u8>) -> Self {
        let (audio_sender, audio_receiver) = mpsc::channel();
        let (output_sender, output_receiver) = mpsc::channel();
        let (interpreter_sender, interpreter_receiver) = mpsc::channel();

        let event_manager = EventRedirectManager::new(vec![
            (audio_sender, [
                IncomingEvent::SetSpeed(0),
                IncomingEvent::Audio(AudioEvent::Any)
            ].to_vec()),
            (output_sender, [
                IncomingEvent::Screen(ScreenEvent::Any),
                IncomingEvent::App(AppEvent::Any)
            ].to_vec()),
            (interpreter_sender, [
                IncomingEvent::Pause(false),
                IncomingEvent::Input(InputEvent::Any),
                IncomingEvent::RequestTermination,
                IncomingEvent::SetSpeed(0),
                IncomingEvent::Interpreter(InterpreterEvent::Any)
            ].to_vec()),
        ]);
        let app_state_event_sender = event_manager.get_event_sender();

        let mut interpreter = Chip8Interpreter::new(Output::new(
            64,
            32,
            10,
            output_receiver,
            audio_receiver,
            app_state_event_sender.clone()
        ));
        interpreter.load_memory(FONT.to_vec(), 0);
        interpreter.load_memory(opcodes, 0x200);
        Self {
            interpreter,
            fps,
            opcodes_per_frame,
            last_opcode: SystemTime::now(),
            opcode_counter: 0,
            interpreter_receiver,
            app_state_event_sender
        }
    }

    pub fn run(&mut self) {
        let mut pressed_keys = HashSet::new();
        let mut pause = false;
        let mut speed = 1.0;
        let mut last_frame = SystemTime::now();
        let millis_between_frames = (1_000_000.0 / self.fps) as u128;
        'main: loop {
            if last_frame.elapsed().unwrap().as_micros() > millis_between_frames {
                self.app_state_event_sender.send(IncomingEvent::Screen(ScreenEvent::Update)).elog("updating");
                last_frame = SystemTime::now();
            }
            while let Ok(e) = self.interpreter_receiver.try_recv() {
                match e {
                    IncomingEvent::Pause(p) => pause = p,
                    IncomingEvent::Input(i_e) => {
                        match i_e {
                            InputEvent::KeyDown(k) => pressed_keys.insert(k).into_empty(),
                            InputEvent::KeyUp(k) => pressed_keys.remove(&k).into_empty(),
                            InputEvent::ClearKeys => pressed_keys.clear(),
                            _ => {}
                        }
                    }, IncomingEvent::RequestTermination => break 'main,
                    IncomingEvent::SetSpeed(s) => speed = SPEED_CHANGE.powi(s as i32),
                    IncomingEvent::Interpreter(i_e) => {
                        match i_e {
                            InterpreterEvent::SetPixel(x, y, c) => self.interpreter.get_output().set(x, y, c.into_bool()),
                            InterpreterEvent::RedrawAll => self.interpreter.get_output().redraw_all(),
                            _ => {}
                        }
                    }, _ => {}
                }
            }
            if pause {
                continue;
            }

            let wait_time = (1_000_000.0 / self.fps / self.opcodes_per_frame as f32 / speed) as u64;
            while wait_time > self.last_opcode.elapsed().unwrap().as_micros() as u64 {};
            self.last_opcode = SystemTime::now();

            self.interpreter.interpret_next(&pressed_keys);

            if self.opcode_counter % self.opcodes_per_frame == 0 {
                self.interpreter.next_frame();
            }
            self.opcode_counter += 1;
        }
    }
}

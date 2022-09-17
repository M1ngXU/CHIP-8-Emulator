use std::collections::HashSet;
use std::fs::{read, write};
use std::path::Path;
use std::sync::mpsc;
use std::time::SystemTime;

use fixed_bit_numbers::IntoEmpty;

use crate::emulator::interpreter::{Chip8Interpreter, Interpreter, InterpreterEvent};
use crate::events::app::AppEvent;
use crate::events::input::InputEvent;
use crate::events::EventRedirectManager;
use crate::sdl2_interaction::audio_manager::AudioEvent;
use crate::sdl2_interaction::event_manager::{AppEventReceiver, AppEventSender, IncomingEvent};
use crate::sdl2_interaction::output::{Output, ScreenEvent};
use crate::sdl2_interaction::screen::Chip8ColorToBool;
use crate::{
    get_fd, LogError, LogWarning, SCREEN_HEIGHT, SCREEN_WIDTH, SPEED_CHANGE_PER_KEYPRESS,
    STARTING_SCALE,
};

mod fixed_bit_numbers;
pub mod interpreter;

static FONT: [u8; 80] = [
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

static DOUBLE_SIZE_FONT: [u8; 160] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator<T: Interpreter> {
    fps: f32,
    opcodes_per_frame: u32,
    interpreter: T,
    last_opcode: SystemTime,
    opcode_counter: u32,
    interpreter_receiver: AppEventReceiver,
    app_state_event_sender: AppEventSender,
}
impl Emulator<Chip8Interpreter> {
    pub fn new_chip8(fps: f32, opcodes_per_frame: u32) -> Self {
        let (audio_sender, audio_receiver) = mpsc::channel();
        let (output_sender, output_receiver) = mpsc::channel();
        let (interpreter_sender, interpreter_receiver) = mpsc::channel();

        let event_manager = EventRedirectManager::new(vec![
            (
                audio_sender,
                [
                    IncomingEvent::SetSpeed(0),
                    IncomingEvent::Audio(AudioEvent::Any),
                ]
                .to_vec(),
            ),
            (
                output_sender,
                [
                    IncomingEvent::Screen(ScreenEvent::Any),
                    IncomingEvent::App(AppEvent::Any),
                    IncomingEvent::Pause(true),
                ]
                .to_vec(),
            ),
            (
                interpreter_sender,
                [
                    IncomingEvent::Pause(false),
                    IncomingEvent::Input(InputEvent::Any),
                    IncomingEvent::RequestTermination,
                    IncomingEvent::SetSpeed(0),
                    IncomingEvent::Interpreter(InterpreterEvent::Any),
                    IncomingEvent::Restart,
                    IncomingEvent::NewGame,
                ]
                .to_vec(),
            ),
        ]);
        let app_state_event_sender = event_manager.get_event_sender();

        let mut interpreter = Chip8Interpreter::new(Output::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            STARTING_SCALE,
            output_receiver,
            audio_receiver,
            app_state_event_sender.clone(),
        ));
        interpreter.load_memory(FONT.to_vec(), 0);
        interpreter.load_memory(DOUBLE_SIZE_FONT.to_vec(), 80);
        Self {
            interpreter,
            fps,
            opcodes_per_frame,
            last_opcode: SystemTime::now(),
            opcode_counter: 0,
            interpreter_receiver,
            app_state_event_sender,
        }
    }

    pub fn load_memory(&mut self, opcodes: Vec<u8>) {
        self.interpreter.reset();
        self.interpreter.load_memory(opcodes, 0x200);
        self.interpreter.get_output().redraw_all();
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) {
        write(&path, self.interpreter.save())
            .elog(format!("saving to {:?}", path.as_ref()).as_str());
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        if !path.as_ref().exists() {
            "Couldn't find quicksaves.".wlog();
            return;
        }
        self.interpreter.load(read(path).unwrap());
        self.interpreter.get_output().redraw_all();
        self.app_state_event_sender
            .send(IncomingEvent::Pause(true))
            .elog("sending pause after load");
    }

    pub fn run(&mut self) -> End {
        self.app_state_event_sender
            .send(IncomingEvent::Pause(false))
            .elog("sending unpause before run");
        let mut pressed_keys = HashSet::new();
        let mut pause = false;
        let mut speed = 1.0;
        let mut last_frame = SystemTime::now();
        let millis_between_frames = (1_000_000.0 / self.fps) as u128;
        if std::fs::read_dir("./saves").is_err() {
            std::fs::create_dir("./saves").elog("creating save directory");
        }
        'main: loop {
            if last_frame.elapsed().unwrap().as_micros() > millis_between_frames {
                self.app_state_event_sender
                    .send(IncomingEvent::Screen(ScreenEvent::Update))
                    .elog("updating");
                last_frame = SystemTime::now();
            }
            while let Ok(e) = self.interpreter_receiver.try_recv() {
                match e {
                    IncomingEvent::Restart => return End::Restart,
                    IncomingEvent::NewGame => return End::NewGame,
                    IncomingEvent::Pause(p) => pause = p,
                    IncomingEvent::Input(i_e) => match i_e {
                        InputEvent::KeyDown(k) => pressed_keys.insert(k).into_empty(),
                        InputEvent::KeyUp(k) => pressed_keys.remove(&k).into_empty(),
                        InputEvent::ClearKeys => pressed_keys.clear(),
                        _ => {}
                    },
                    IncomingEvent::RequestTermination => break 'main,
                    IncomingEvent::SetSpeed(s) => speed = SPEED_CHANGE_PER_KEYPRESS.powi(s as i32),
                    IncomingEvent::Interpreter(i_e) => match i_e {
                        InterpreterEvent::SetPixel(x, y, c) => {
                            let scale = self.interpreter.get_output().get_screen().get_scale();
                            self.interpreter.get_output_mut().set(
                                x / scale,
                                y / scale,
                                c.into_bool(),
                            );
                        }
                        InterpreterEvent::RedrawAll => {
                            self.interpreter.get_output_mut().redraw_all()
                        }
                        InterpreterEvent::QuickSave => self.save(
                            format!(
                                "./saves/quicksave-{}.ch8",
                                SystemTime::now()
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs()
                            )
                            .as_str(),
                        ),
                        InterpreterEvent::QuickLoad => self.load(
                            format!(
                                "./saves/{}",
                                std::fs::read_dir("./saves/")
                                    .unwrap()
                                    .filter_map(|f| f.ok())
                                    .filter_map(|f| f.file_name().into_string().ok())
                                    .reduce(|a, b| {
                                        if a.starts_with("quicksave-")
                                            && b.starts_with("quicksave-")
                                            && a.cmp(&b).is_lt()
                                        {
                                            b
                                        } else {
                                            a
                                        }
                                    })
                                    .unwrap()
                            )
                            .as_str(),
                        ),
                        InterpreterEvent::Save => {
                            if let Some(path) = get_fd("saves")
                                .set_file_name("quicksave-untitled.ch8-save")
                                .add_filter("Chip8 Save", &["ch8-save"])
                                .add_filter("all", &["*"])
                                .set_title("Choose a save location.")
                                .save_file()
                            {
                                self.save(path);
                                self.app_state_event_sender
                                    .send(IncomingEvent::Pause(false))
                                    .elog("resuming emulation");
                            }
                        }
                        InterpreterEvent::Load => {
                            if let Some(path) = get_fd("saves")
                                .add_filter("Chip8 Save", &["ch8-save"])
                                .add_filter("all", &["*"])
                                .set_title("Choose a Chip8 Save to load")
                                .pick_file()
                            {
                                self.load(path);
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            if pause {
                continue;
            }

            let wait_time = (1_000_000.0 / self.fps / self.opcodes_per_frame as f32 / speed) as u64;
            while wait_time > self.last_opcode.elapsed().unwrap().as_micros() as u64 {}
            self.last_opcode = SystemTime::now();

            self.interpreter.interpret_next(&pressed_keys);

            if self.opcode_counter % self.opcodes_per_frame == 0 {
                self.interpreter.next_frame();
            }
            self.opcode_counter += 1;
        }
        End::Quit
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum End {
    Quit,
    Restart,
    NewGame,
}

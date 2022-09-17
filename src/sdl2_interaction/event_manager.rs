use std::collections::HashSet;
use std::mem::discriminant;
use std::sync::mpsc;

use sdl2::event::{Event as SdlEvent, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;

use crate::emulator::interpreter::InterpreterEvent;
use crate::events::app::AppEvent;
use crate::events::input::InputEvent;
use crate::sdl2_interaction::audio_manager::AudioEvent;
use crate::sdl2_interaction::output::ScreenEvent;
use crate::LogError;

pub type AppEventSender = mpsc::Sender<IncomingEvent>;
pub type AppEventReceiver = mpsc::Receiver<IncomingEvent>;

pub trait Event {
    fn is_any(&self) -> bool;
    fn compare_value(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        discriminant(self) == discriminant(other)
    }
    fn equals(&self, other: &Self) -> bool
    where
        Self: Sized,
    {
        self.is_any() || other.is_any() || self.compare_value(other)
    }
}

#[derive(Clone, Debug)]
pub enum IncomingEvent {
    RequestTermination,
    Pause(bool),
    SetSpeed(i8),
    Input(InputEvent),
    Interpreter(InterpreterEvent),
    Screen(ScreenEvent),
    Audio(AudioEvent),
    App(AppEvent),
    SetCheatMode(bool),
    Restart,
    NewGame,
    Any,
}

impl Event for IncomingEvent {
    fn is_any(&self) -> bool {
        matches!(&self, &IncomingEvent::Any)
    }
    fn compare_value(&self, other: &Self) -> bool {
        match (self, other) {
            (IncomingEvent::App(s), IncomingEvent::App(o)) => s.equals(o),
            (IncomingEvent::Input(s), IncomingEvent::Input(o)) => s.equals(o),
            (IncomingEvent::Interpreter(s), IncomingEvent::Interpreter(o)) => s.equals(o),
            (IncomingEvent::Screen(s), IncomingEvent::Screen(o)) => s.equals(o),
            (IncomingEvent::Audio(s), IncomingEvent::Audio(o)) => s.equals(o),
            _ => discriminant(self) == discriminant(other),
        }
    }
}

pub struct AppEventManager {
    app_event_sender: AppEventSender,
    pressed_keys: HashSet<Scancode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
}

impl AppEventManager {
    pub fn new(app_event_sender: AppEventSender) -> Self {
        Self {
            app_event_sender,
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
        }
    }

    /// multithreading possible
    pub fn update(&mut self, event: SdlEvent, scale: (u32, u32)) {
        for key in self.pressed_keys.iter() {
            self.app_event_sender
                .send(IncomingEvent::Input(InputEvent::KeyPress(*key)))
                .elog("sending key press");
        }
        for mouse_button in self.pressed_mouse_buttons.iter() {
            self.app_event_sender
                .send(IncomingEvent::Input(InputEvent::MouseButtonPress(
                    *mouse_button,
                )))
                .elog("sending mouse button press");
        }
        self.app_event_sender
            .send(match event {
                SdlEvent::KeyDown {
                    scancode: Some(s),
                    repeat: false,
                    ..
                } => {
                    self.pressed_keys.insert(s);
                    IncomingEvent::Input(InputEvent::KeyDown(s))
                }
                SdlEvent::KeyUp {
                    scancode: Some(s),
                    repeat: false,
                    ..
                } => {
                    self.pressed_keys.remove(&s);
                    IncomingEvent::Input(InputEvent::KeyUp(s))
                }
                SdlEvent::Window {
                    win_event: WindowEvent::FocusLost,
                    ..
                } => {
                    self.pressed_keys.clear();
                    self.app_event_sender
                        .send(IncomingEvent::Input(InputEvent::ClearKeys))
                        .elog("clearing keys");
                    self.pressed_mouse_buttons.clear();
                    self.app_event_sender
                        .send(IncomingEvent::Input(InputEvent::ClearMouseButtons))
                        .elog("clearing mouse buttons");
                    IncomingEvent::App(AppEvent::SetFocus(false))
                }
                SdlEvent::Window {
                    win_event: WindowEvent::FocusGained,
                    ..
                } => IncomingEvent::App(AppEvent::SetFocus(true)),
                SdlEvent::MouseButtonDown { mouse_btn, .. }
                    if self.pressed_mouse_buttons.insert(mouse_btn) =>
                {
                    IncomingEvent::Input(InputEvent::MouseButtonDown(mouse_btn))
                }
                SdlEvent::MouseButtonUp { mouse_btn, .. }
                    if self.pressed_mouse_buttons.remove(&mouse_btn) =>
                {
                    IncomingEvent::Input(InputEvent::MouseButtonUp(mouse_btn))
                }
                SdlEvent::MouseMotion { x, y, .. } => IncomingEvent::Input(
                    InputEvent::UpdateMouseCoordinates(x / scale.0 as i32, y / scale.1 as i32),
                ),
                SdlEvent::Window {
                    win_event: WindowEvent::SizeChanged(w, h),
                    ..
                } => IncomingEvent::App(AppEvent::WindowSizeChange(w, h)),
                SdlEvent::Window {
                    win_event: WindowEvent::Close,
                    ..
                } => IncomingEvent::RequestTermination,
                _ => return,
            })
            .elog("updating");
    }
}

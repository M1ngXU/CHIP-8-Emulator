use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;

use crate::emulator::interpreter::InterpreterEvent;
use crate::events::EventManager;
use crate::sdl2_interaction::event_manager::{Event, IncomingEvent};
use crate::sdl2_interaction::output::ScreenEvent;
use crate::sdl2_interaction::screen::Chip8BoolToColor;
use crate::LogWarning;

#[derive(Copy, Clone, Debug)]
pub enum InputEvent {
    KeyPress(Scancode),
    KeyDown(Scancode),
    KeyUp(Scancode),
    ClearKeys,
    MouseButtonDown(MouseButton),
    MouseButtonPress(MouseButton),
    MouseButtonUp(MouseButton),
    ClearMouseButtons,
    UpdateMouseCoordinates(i32, i32),
    Any,
}
impl Event for InputEvent {
    fn is_any(&self) -> bool {
        matches!(&self, &InputEvent::Any)
    }
}
pub struct InputEventManager {
    pause_state: bool,
    speed: i8,
    is_in_cheat_mode: bool,
    mouse_coordinates: (usize, usize),
}
impl EventManager for InputEventManager {
    fn new() -> Self {
        Self {
            pause_state: false,
            speed: 0,
            is_in_cheat_mode: false,
            mouse_coordinates: (0, 0),
        }
    }

    fn update(&mut self, event: &IncomingEvent) -> Option<IncomingEvent> {
        match event {
            IncomingEvent::SetSpeed(s) => self.speed = *s,
            IncomingEvent::Pause(p) => self.pause_state = *p,
            IncomingEvent::SetCheatMode(c) => self.is_in_cheat_mode = *c,
            IncomingEvent::Input(InputEvent::UpdateMouseCoordinates(x, y)) => {
                self.mouse_coordinates = (*x as usize, *y as usize)
            }
            _ => {
                return Some(match event {
                    IncomingEvent::Input(InputEvent::MouseButtonPress(m))
                        if self.is_in_cheat_mode =>
                    {
                        IncomingEvent::Interpreter(InterpreterEvent::SetPixel(
                            self.mouse_coordinates.0,
                            self.mouse_coordinates.1,
                            match m {
                                MouseButton::Left => true,
                                MouseButton::Right => false,
                                _ => return None,
                            }
                            .into_color(),
                        ))
                    }
                    IncomingEvent::Input(InputEvent::KeyDown(k)) => match k {
                        Scancode::F1 => IncomingEvent::SetSpeed(0),
                        Scancode::F2 => {
                            if self.speed > i8::MIN {
                                IncomingEvent::SetSpeed(self.speed - 1)
                            } else {
                                "Reached min speed!".wlog();
                                return None;
                            }
                        }
                        Scancode::F3 => {
                            if self.speed < i8::MAX {
                                IncomingEvent::SetSpeed(self.speed + 1)
                            } else {
                                "Reached max speed!".wlog();
                                return None;
                            }
                        }
                        Scancode::F4 => IncomingEvent::SetCheatMode(!self.is_in_cheat_mode),
                        Scancode::F5 => IncomingEvent::Interpreter(InterpreterEvent::QuickSave),
                        Scancode::F6 => IncomingEvent::Restart,
                        Scancode::F7 => IncomingEvent::NewGame,
                        Scancode::F8 => IncomingEvent::Interpreter(InterpreterEvent::QuickLoad),
                        Scancode::F9 => IncomingEvent::Interpreter(InterpreterEvent::Save),
                        Scancode::F10 => IncomingEvent::Interpreter(InterpreterEvent::Load),
                        Scancode::F11 => IncomingEvent::Screen(ScreenEvent::ToggleFullscreen),
                        Scancode::Escape => IncomingEvent::Pause(!self.pause_state),
                        _ => return None,
                    },
                    _ => return None,
                });
            }
        }
        None
    }

    fn get_callbacks(&self) -> &[IncomingEvent] {
        &[
            IncomingEvent::Pause(false),
            IncomingEvent::Input(InputEvent::Any),
            IncomingEvent::SetSpeed(0),
            IncomingEvent::SetCheatMode(true),
        ]
    }
}

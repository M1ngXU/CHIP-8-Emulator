use crate::emulator::interpreter::InterpreterEvent;
use crate::events::EventManager;
use crate::sdl2_interaction::event_manager::{Event, IncomingEvent};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum AppEvent {
    WindowSizeChange(i32, i32),
    SetFocus(bool),
    Any
}
impl Event for AppEvent {
    fn is_any(&self) -> bool {
        matches!(&self, &AppEvent::Any)
    }
}
pub struct AppEventManager {}

impl EventManager for AppEventManager {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, event: &IncomingEvent) -> Option<IncomingEvent> {
        Some(match event {
            IncomingEvent::App(AppEvent::WindowSizeChange(_, _)) => IncomingEvent::Interpreter(InterpreterEvent::RedrawAll),
            IncomingEvent::App(AppEvent::SetFocus(f)) => {
                if *f {
                    IncomingEvent::Interpreter(InterpreterEvent::RedrawAll)
                } else {
                    IncomingEvent::Pause(true)
                }
            },
            _ => return None
        })
    }

    fn get_callbacks(&self) -> &[IncomingEvent] {
        &[ IncomingEvent::App(AppEvent::Any) ]
    }
}
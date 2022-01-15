use crate::{LogInfo, LogWarning, SPEED_CHANGE_PER_KEYPRESS};
use crate::events::EventManager;
use crate::sdl2_interaction::event_manager::IncomingEvent;

pub struct Logger {}
impl EventManager for Logger {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, event: &IncomingEvent) -> Option<IncomingEvent> {
        match event {
            IncomingEvent::Pause(p) => {
                if *p {
                    "Paused emulation.".log();
                } else {
                    "Un-paused emulation.".log();
                }
            }, IncomingEvent::SetCheatMode(c) => {
                if *c {
                    "Cheat mode turned on. Draw onto the screen to modify collisions.".wlog();
                } else {
                    "Cheat mode turned off.".wlog();
                }
            }, IncomingEvent::SetSpeed(s) => format!("Changed speed to {}%.", (SPEED_CHANGE_PER_KEYPRESS.powi(*s as i32) * 100.0) as i32).as_str().log(),
            _ => {}
        }
       None
    }

    fn get_callbacks(&self) -> &[ IncomingEvent ] {
        &[ IncomingEvent::Any ]
    }
}
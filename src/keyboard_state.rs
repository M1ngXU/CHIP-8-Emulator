use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use sdl2::keyboard::Scancode;
use crate::event_manager::EventManager;

static KEYBOARD_LAYOUT: [ Scancode; 16 ] = [
    Scancode::Kp1, Scancode::Kp2, Scancode::Kp3, Scancode::Kp4,
    Scancode::Q, Scancode::W, Scancode::E, Scancode::R,
    Scancode::A, Scancode::S, Scancode::D, Scancode::F,
    Scancode::Z, Scancode::X, Scancode::C, Scancode::V
];
static HEX_LAYOUT: [ u8; 16 ] = [
    0x1, 0x2, 0x3, 0xC,
    0x4, 0x5, 0x6, 0xD,
    0x7, 0x8, 0x9, 0xE,
    0xA, 0x0, 0xB, 0xF
];
/// Wrapper struct for `Scancode` from SDL2
#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct PressedKey {
    scancode: Scancode
}
impl PressedKey {
    pub fn new(scancode: Scancode) -> Self {
        Self {
            scancode
        }
    }

    pub fn to_hex(&self) -> Option<u8> {
        KEYBOARD_LAYOUT.iter()
            .position(| &s | s == self.scancode)
            .and_then(| i | Some(HEX_LAYOUT[i]))
    }

    pub fn from_hex(hex: u8) -> Option<Scancode> {
        HEX_LAYOUT.iter()
            .position(| &n | n == hex)
            .and_then(| i | Some(KEYBOARD_LAYOUT[i]))
    }
}
impl Into<PressedKey> for Scancode {
    fn into(self) -> PressedKey {
        PressedKey::new(self)
    }
}

pub struct KeyState {
    event_manager: Arc<Mutex<EventManager>>,
    pub currently_pressed: HashSet<PressedKey>,
    pub just_released: HashSet<PressedKey>,
    pub just_pressed: HashSet<PressedKey>,
    pub last_update: SystemTime
}
impl KeyState {
    pub fn new(event_manager: Arc<Mutex<EventManager>>) -> Self {
        Self {
            event_manager,
            currently_pressed: HashSet::new(),
            just_released: HashSet::new(),
            just_pressed: HashSet::new(),
            last_update: SystemTime::now()
        }
    }

    pub fn update(&mut self) {
        self.last_update = SystemTime::now();
        let pressed = self.event_manager.lock().unwrap().get_pressed_keys().clone();
        self.just_released = self.currently_pressed.difference(&pressed).map(| p | *p).collect();
        self.just_pressed = pressed.difference(&self.currently_pressed).map(| p | *p).collect();
        self.currently_pressed = pressed;
    }
}
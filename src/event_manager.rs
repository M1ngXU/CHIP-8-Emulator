use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use sdl2::event::{Event, EventPollIterator};
use sdl2::keyboard::Scancode;

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

pub struct EventManager {
    keys_pressed: Arc<Mutex<HashSet<PressedKey>>>,
    terminating: Arc<Mutex<bool>>
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            keys_pressed: Arc::new(Mutex::new(HashSet::new())),
            terminating: Arc::new(Mutex::new(false))
        }
    }

    // multithreading possible
    pub fn update(&mut self, recent_events: EventPollIterator) {
        for event in recent_events {
            match event {
                Event::KeyDown { scancode: Some(s), .. } => {
                    self.keys_pressed.lock().unwrap().insert(PressedKey::new(s));
                }, Event::KeyUp { scancode: Some(s), .. } => {
                    self.keys_pressed.lock().unwrap().remove(&PressedKey::new(s));
                }, Event::AppDidEnterBackground { .. } | Event::AppDidEnterForeground { .. } => {
                    self.keys_pressed.lock().unwrap().clear();
                }, Event::Quit { .. } => {
                    *self.terminating.lock().unwrap() = true;
                }, _ => {}
            };
        }
    }

    fn get_pressed_keys(&self) -> MutexGuard<HashSet<PressedKey>> {
        self.keys_pressed.lock().unwrap()
    }

    pub fn is_terminating(&self) -> bool {
        *self.terminating.lock().unwrap()
    }

    pub fn is_scancode_pressed(&self, s: Scancode) -> bool {
        self.is_key_pressed(PressedKey::new(s))
    }

    pub fn is_key_pressed(&self, k: PressedKey) -> bool {
        self.get_pressed_keys().contains(&k)
    }

    pub fn get_one_pressed_key(&self) -> Option<PressedKey> {
        self.get_pressed_keys().iter().next().and_then(| s | Some(*s))
    }
}
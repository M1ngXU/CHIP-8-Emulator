use std::collections::HashSet;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
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

    pub fn to_hex(self) -> Option<u8> {
        KEYBOARD_LAYOUT.iter()
            .position(| &s | s == self.scancode)
            .map(| i | HEX_LAYOUT[i])
    }

    pub fn from_hex(hex: u8) -> Option<PressedKey> {
        HEX_LAYOUT.iter()
            .position(| &n | n == hex)
            .map(| i | KEYBOARD_LAYOUT[i].into())
    }
}
impl From<Scancode> for PressedKey {
    fn from(scancode: Scancode) -> PressedKey {
        PressedKey::new(scancode)
    }
}

pub trait InputState<T> {
    fn new() -> Self;

    fn update(&mut self, current: HashSet<T>);

    fn just_released(&self, i: T) -> bool;
    fn just_released_all(&self) -> HashSet<T>;

    fn just_pressed(&self, i: T) -> bool;
    fn just_pressed_all(&self) -> HashSet<T>;

    fn currently_pressed(&self, i: T) -> bool;
    fn currently_pressed_all(&self) -> HashSet<T>;
}
pub struct KeyboardMouseState<T> {
    just_released: HashSet<T>,
    just_pressed: HashSet<T>,
    currently_pressed: HashSet<T>
}
impl<T: Copy + Eq + Hash> InputState<T> for KeyboardMouseState<T> {
    fn new() -> Self {
        Self {
            just_released: HashSet::new(),
            just_pressed: HashSet::new(),
            currently_pressed: HashSet::new()
        }
    }

    fn update(&mut self, current: HashSet<T>) {
        self.just_released = self.currently_pressed.difference(&current).copied().collect();
        self.just_pressed = current.difference(&self.currently_pressed).copied().collect();
        self.currently_pressed = current;
    }

    fn just_released(&self, i: T) -> bool {
        self.just_released.contains(&i)
    }

    fn just_released_all(&self) -> HashSet<T> {
        self.just_released.clone()
    }

    fn just_pressed(&self, i: T) -> bool {
        self.just_pressed.contains(&i)
    }

    fn just_pressed_all(&self) -> HashSet<T> {
        self.just_pressed.clone()
    }

    fn currently_pressed(&self, i: T) -> bool {
        self.currently_pressed.contains(&i)
    }

    fn currently_pressed_all(&self) -> HashSet<T> {
        self.currently_pressed.clone()
    }
}

pub struct AppState {
    event_manager: Arc<Mutex<EventManager>>,
    mouse_button_state: KeyboardMouseState<MouseButton>,
    keyboard_state: KeyboardMouseState<PressedKey>,
    mouse_coordinates: (i32, i32),
    lost_focus: bool,
    focused: bool,
    gained_focus: bool,
    is_terminating: bool
}
impl AppState {
    pub fn new(event_manager: Arc<Mutex<EventManager>>) -> Self {
        Self {
            event_manager,
            mouse_button_state: KeyboardMouseState::new(),
            keyboard_state: KeyboardMouseState:: new(),
            mouse_coordinates: (0, 0),
            lost_focus: false,
            gained_focus: false,
            focused: true,
            is_terminating: false
        }
    }

    pub fn update(&mut self) {
        let em = self.event_manager.lock().unwrap();

        self.keyboard_state.update(em.get_pressed_keys().clone());
        self.mouse_button_state.update(em.get_pressed_mouse_buttons().clone());

        self.mouse_coordinates = *em.get_mouse_coordinates();

        self.lost_focus = self.is_focused() && !em.is_focused();
        self.gained_focus = !self.is_focused() && em.is_focused();
        self.focused = em.is_focused();

        self.is_terminating = em.is_terminating();
    }

    pub fn get_keyboard_state(&self) -> &KeyboardMouseState<PressedKey> {
        &self.keyboard_state
    }

    pub fn get_mouse_button_state(&self) -> &KeyboardMouseState<MouseButton> {
        &self.mouse_button_state
    }

    pub fn just_lost_focus(&self) -> bool {
        self.lost_focus
    }

    pub fn just_gained_focus(&self) -> bool {
        self.gained_focus
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_terminating(&self) -> bool {
        self.is_terminating
    }

    pub fn get_mouse_coordinates(&self) -> (i32, i32) {
        self.mouse_coordinates
    }
}
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use sdl2::mouse::MouseButton;
use crate::event_manager::EventManager;

pub struct MouseState {
    event_manager: Arc<Mutex<EventManager>>,
    pub currently_pressed: HashSet<MouseButton>,
    pub just_released: HashSet<MouseButton>,
    pub just_pressed: HashSet<MouseButton>,
    pub coordinates: (i32, i32),
    pub last_update: SystemTime
}
impl MouseState {
    pub fn new(event_manager: Arc<Mutex<EventManager>>) -> Self {
        Self {
            event_manager,
            currently_pressed: HashSet::new(),
            just_released: HashSet::new(),
            just_pressed: HashSet::new(),
            coordinates: (0, 0),
            last_update: SystemTime::now()
        }
    }

    pub fn update(&mut self) {
        self.last_update = SystemTime::now();
        let em = self.event_manager.lock().unwrap();
        let pressed = em.get_pressed_mouse_buttons().clone();
        self.just_released = self.currently_pressed.difference(&pressed).map(| p | *p).collect();
        self.just_pressed = pressed.difference(&self.currently_pressed).map(| p | *p).collect();
        self.currently_pressed = pressed;
        self.coordinates = *em.get_mouse_coordinates();
    }
}
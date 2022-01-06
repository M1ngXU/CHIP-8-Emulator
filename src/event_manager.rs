use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use sdl2::event::{Event, EventPollIterator, WindowEvent};
use sdl2::mouse::MouseButton;
use crate::keyboard_state::PressedKey;

pub struct EventManager {
    keys_pressed: Arc<Mutex<HashSet<PressedKey>>>,
    mouse_buttons_pressed: Arc<Mutex<HashSet<MouseButton>>>,
    mouse_coordinates: Arc<Mutex<(i32, i32)>>,
    terminating: Arc<Mutex<bool>>,
    focused: Arc<Mutex<bool>>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            keys_pressed: Arc::new(Mutex::new(HashSet::new())),
            mouse_buttons_pressed: Arc::new(Mutex::new(HashSet::new())),
            mouse_coordinates: Arc::new(Mutex::new((0, 0))),
            terminating: Arc::new(Mutex::new(false)),
            focused: Arc::new(Mutex::new(true))
        }
    }

    // multithreading possible
    pub fn update(&mut self, recent_events: EventPollIterator) {
        for event in recent_events {
            match event {
                Event::Window { win_event: WindowEvent::Close, .. } => {
                    *self.terminating.lock().unwrap() = true;
                    log::info!("Terminating...");
                }, _ => {
                    if self.is_focused() {
                        match event {
                            Event::KeyDown { scancode: Some(s), .. } => {
                                self.keys_pressed.lock().unwrap().insert(s.into());
                            }, Event::KeyUp { scancode: Some(s), .. } => {
                                self.keys_pressed.lock().unwrap().remove(&s.into());
                            }, Event::Window { win_event: WindowEvent::FocusLost, .. } => {
                                self.keys_pressed.lock().unwrap().clear();
                                self.mouse_buttons_pressed.lock().unwrap().clear();
                                *self.focused.lock().unwrap() = false;
                                log::info!("Lost focus.");
                            }, Event::MouseButtonDown { mouse_btn, .. } => {
                                self.mouse_buttons_pressed.lock().unwrap().insert(mouse_btn);
                            }, Event::MouseButtonUp { mouse_btn, .. } => {
                                self.mouse_buttons_pressed.lock().unwrap().remove(&mouse_btn);
                            }, Event::MouseMotion { x, y, .. } => {
                                *self.mouse_coordinates.lock().unwrap() = (x, y);
                            }, _ => {}
                        }
                    } else {
                        match event {
                            Event::Window { win_event: WindowEvent::FocusGained, .. } => {
                                *self.focused.lock().unwrap() = true;
                                log::info!("Regained focus.");
                            }, _ => {}
                        }
                    }
                }
            }
        }
    }

    pub fn get_pressed_keys(&self) -> MutexGuard<HashSet<PressedKey>> {
        self.keys_pressed.lock().unwrap()
    }

    pub fn get_pressed_mouse_buttons(&self) -> MutexGuard<HashSet<MouseButton>> {
        self.mouse_buttons_pressed.lock().unwrap()
    }

    pub fn get_mouse_coordinates(&self) -> MutexGuard<(i32, i32)> {
        self.mouse_coordinates.lock().unwrap()
    }

    pub fn is_focused(&self) -> bool {
        *self.focused.lock().unwrap()
    }

    pub fn is_terminating(&self) -> bool {
        *self.terminating.lock().unwrap()
    }

    pub fn is_key_pressed(&self, k: PressedKey) -> bool {
        self.get_pressed_keys().contains(&k)
    }
}
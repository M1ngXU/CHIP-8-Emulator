use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use sdl2::event::{Event, WindowEvent};
use sdl2::mouse::MouseButton;
use crate::app_state::PressedKey;
use crate::logger::LogInfo;

pub struct EventManager {
    keys_pressed: Arc<Mutex<HashSet<PressedKey>>>,
    mouse_buttons_pressed: Arc<Mutex<HashSet<MouseButton>>>,
    mouse_coordinates: Arc<Mutex<(i32, i32)>>,
    terminating: Arc<AtomicBool>,
    focused: Arc<AtomicBool>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            keys_pressed: Arc::new(Mutex::new(HashSet::new())),
            mouse_buttons_pressed: Arc::new(Mutex::new(HashSet::new())),
            mouse_coordinates: Arc::new(Mutex::new((0, 0))),
            terminating: Arc::new(AtomicBool::new(false)),
            focused: Arc::new(AtomicBool::new(true))
        }
    }

    // multithreading possible
    pub fn update(&mut self, event: Event, scale: (u32, u32)) {
        match event {
            Event::Window { win_event: WindowEvent::Close, .. } => {
                self.terminating.store(true, Ordering::Relaxed);
                "Terminating...".log();
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
                            self.focused.store(false, Ordering::Relaxed);
                            "Lost focus.".log()
                        }, Event::MouseButtonDown { mouse_btn, .. } => {
                            self.mouse_buttons_pressed.lock().unwrap().insert(mouse_btn);
                        }, Event::MouseButtonUp { mouse_btn, .. } => {
                            self.mouse_buttons_pressed.lock().unwrap().remove(&mouse_btn);
                        }, Event::MouseMotion { x, y, .. } => {
                            *self.mouse_coordinates.lock().unwrap() = (x / scale.0 as i32, y / scale.1 as i32);
                        }, _ => {}
                    }
                } else if let Event::Window { win_event: WindowEvent::FocusGained, .. } = event {
                    self.focused.store(true, Ordering::Relaxed);
                    "Regained focus.".log();
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
        self.focused.load(Ordering::Relaxed)
    }

    pub fn is_terminating(&self) -> bool {
        self.terminating.load(Ordering::Relaxed)
    }

    pub fn is_key_pressed(&self, k: PressedKey) -> bool {
        self.get_pressed_keys().contains(&k)
    }
}
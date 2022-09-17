use std::collections::VecDeque;
use std::sync::mpsc;
use std::thread;

use crate::events::app::AppEventManager;
use crate::events::input::InputEventManager;
use crate::events::logger::Logger;
use crate::sdl2_interaction::event_manager::{AppEventSender, Event, IncomingEvent};
use crate::LogError;

pub mod app;
pub mod input;
mod logger;

pub trait EventManager {
    fn new() -> Self
    where
        Self: Sized;
    fn update(&mut self, app_event: &IncomingEvent) -> Option<IncomingEvent>;
    fn get_callbacks(&self) -> &[IncomingEvent];
}

pub struct EventRedirectManager {
    event_sender: AppEventSender,
}
impl EventRedirectManager {
    pub fn new(callbacks: Vec<(AppEventSender, Vec<IncomingEvent>)>) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut pending_events = VecDeque::new();
            let mut event_managers: [Box<dyn EventManager>; 3] = [
                Box::new(InputEventManager::new()),
                Box::new(AppEventManager::new()),
                Box::new(Logger::new()),
            ];
            loop {
                while let Ok(event) = event_receiver.try_recv() {
                    pending_events.push_back(event);
                }
                while let Some(event) = pending_events.pop_front() {
                    for sender in callbacks
                        .iter()
                        .filter_map(|(s, c)| c.iter().any(|a_e| a_e.equals(&event)).then(|| s))
                    {
                        sender.send(event.clone()).elog("sending event");
                    }
                    for manager in event_managers.iter_mut() {
                        if manager.get_callbacks().iter().any(|e| e.equals(&event)) {
                            if let Some(returned_event) = manager.update(&event) {
                                pending_events.push_back(returned_event);
                            }
                        }
                    }
                }
            }
        });
        Self { event_sender }
    }

    pub fn get_event_sender(&self) -> AppEventSender {
        self.event_sender.clone()
    }
}

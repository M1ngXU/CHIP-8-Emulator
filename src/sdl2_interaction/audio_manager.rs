use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;

use crate::sdl2_interaction::event_manager::{AppEventReceiver, Event, IncomingEvent};
use crate::{SPEED_CHANGE_PER_KEYPRESS, STANDARD_BUZZ_FREQUENCY};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum AudioEvent {
    Buzz(bool),
    Any,
}
impl Event for AudioEvent {
    fn is_any(&self) -> bool {
        matches!(self, &AudioEvent::Any)
    }
}

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}
impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct AudioManager<'a> {
    sdl_context: &'a Sdl,
    current_device: AudioDevice<SquareWave>,
    callback_receiver: AppEventReceiver,
}

impl<'a> AudioManager<'a> {
    pub fn new(sdl_context: &'a Sdl, callback_receiver: AppEventReceiver) -> Self {
        Self {
            sdl_context,
            current_device: Self::get_buzz_device(sdl_context, 0),
            callback_receiver,
        }
    }

    pub fn update(&mut self) {
        while let Ok(event) = self.callback_receiver.try_recv() {
            match event {
                IncomingEvent::SetSpeed(s) => {
                    self.current_device = Self::get_buzz_device(self.sdl_context, s)
                }
                IncomingEvent::Audio(AudioEvent::Buzz(b)) => {
                    if b {
                        self.current_device.resume()
                    } else {
                        self.current_device.pause()
                    }
                }
                _ => {}
            }
        }
    }

    fn get_buzz_device(sdl_context: &Sdl, speed: i8) -> AudioDevice<SquareWave> {
        sdl_context
            .audio()
            .unwrap()
            .open_playback(
                None,
                &AudioSpecDesired {
                    freq: Some(44100),
                    channels: Some(1),
                    samples: None,
                },
                |spec| SquareWave {
                    phase_inc: SPEED_CHANGE_PER_KEYPRESS.powi(speed as i32)
                        * STANDARD_BUZZ_FREQUENCY
                        / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                },
            )
            .unwrap()
    }
}

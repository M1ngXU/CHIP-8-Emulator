use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;

pub struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
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

pub struct AudioManager;

impl AudioManager {
    pub fn get_prepared_buzz_device(sdl_context: &Sdl) -> AudioDevice<SquareWave> {
        sdl_context.audio().unwrap()
            .open_playback(
                None,
                &AudioSpecDesired {
                    freq: Some(44100),
                    channels: Some(1),
                    samples: None
                },
                | spec | SquareWave {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25
                }
            ).unwrap()
    }
}
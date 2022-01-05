use std::sync::{Arc, mpsc, Mutex, MutexGuard};
use std::sync::mpsc::Sender;
use std::thread;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use crate::event_manager::{EventManager, PressedKey};

pub struct Screen {
	pix: Vec<Vec<bool>>,
	width: u32,
	height: u32,
	scale: u32,
	event_manager: Arc<Mutex<EventManager>>,
	sender: Sender<ScreenEvent>
}

enum ScreenEvent {
	Clear,
	Update,
	Set(u32, u32, bool)
}

fn get_sized_vec<T: Copy>(value: T, width: u32, height: u32) -> Vec<Vec<T>> {
	let mut new = Vec::new();
	for _ in 0..height {
		new.push(Vec::from([ value ].repeat(width as usize)));
	}
	new
}

impl Screen {
	pub fn new(width: u32, height: u32, scale: u32) -> Self {
		let (sender, receiver) = mpsc::channel();
		let event_manager = Arc::new(Mutex::new(EventManager::new()));
		let e = event_manager.clone();

		thread::spawn(move || {
			let sdl_context = sdl2::init().unwrap();
			let mut canvas = sdl_context.video().unwrap()
				.window("CHIP 8", width * scale, height * scale)
				.position_centered()
				.build()
				.unwrap()
				.into_canvas().build().unwrap();



			/*let audio_subsystem = sdl_context.audio().unwrap();

			let desired_spec = AudioSpecDesired {
				freq: Some(44100),
				channels: Some(1),  // mono
				samples: None       // default sample size
			};

			let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
				// initialize the audio callback
				SquareWave {
					phase_inc: 440.0 / spec.freq as f32,
					phase: 0.0,
					volume: 0.25
				}
			}).unwrap();

// Start playback
			device.resume();
*/

			canvas.set_logical_size(width, height);
			canvas.set_scale(scale as f32, scale as f32);

			let mut event_pump = sdl_context.event_pump().unwrap();
			loop {
				if let Ok(screen_event) = receiver.try_recv() {
					match screen_event {
						ScreenEvent::Update => canvas.present(),
						ScreenEvent::Clear => {
							canvas.set_draw_color(Color::RGB(0, 0, 0));
							canvas.clear();
						}, ScreenEvent::Set(x, y, v) => {
							if v {
								canvas.set_draw_color(Color::RGB(255, 255, 255))
							} else {
								canvas.set_draw_color(Color::RGB(0, 0, 0));
							}
							canvas.draw_point(Point::new(x as i32, y as i32));
						}
					}
				}
				e.lock().unwrap().update(event_pump.poll_iter());
			}
		});

		let mut new = Self {
			// one line for blocking key request, other line for BUZZ
			pix: get_sized_vec(false, width, height),
			width,
			height,
			scale,
			event_manager,
			sender
		};
		new.clear();
		new
	}

	pub fn get_event_manager(&self) -> MutexGuard<EventManager> {
		self.event_manager.lock().unwrap()
	}

	pub fn get_event_manager_pointer(&self) -> Arc<Mutex<EventManager>> {
		self.event_manager.clone()
	}

	pub fn clear(&mut self) {
		self.pix = get_sized_vec(false, self.width, self.height);
		self.sender.send(ScreenEvent::Clear);
		self.sender.send(ScreenEvent::Update);
	}

	pub fn in_bounds(&self, x: u32, y: u32) -> bool {
		y < self.height && x < self.width
	}

	pub fn get(&self, x: u32, y: u32) -> bool {
		self.in_bounds(x, y) && self.pix[y as usize][x as usize]
	}

	pub fn set(&mut self, x: u32, y: u32, v: bool) {
		if self.in_bounds(x, y) {
			self.pix[y as usize][x as usize] = v;
			self.sender.send(ScreenEvent::Set(x, y, v));
		}
	}

	pub fn swap(&mut self, x: u32, y: u32) {
		self.set(x, y, !self.get(x, y));
	}

	pub fn update(&mut self) {
		self.sender.send(ScreenEvent::Update);
	}

	pub fn is_pressed(&mut self, key: u8) -> bool {
		if let Some(k) = PressedKey::from_hex(key) {
			self.get_event_manager().is_scancode_pressed(k)
		} else {
			false
		}
	}

	pub fn buzz(&self) {
	}

	pub fn get_blocking_input(&mut self) -> u8 {
		loop {
			if let Some(h) = self.get_event_manager()
					.get_one_pressed_key()
					.and_then(| k | k.to_hex()) {
				break h;
			}
		}
	}
}
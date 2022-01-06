use std::sync::{Arc, mpsc, Mutex, MutexGuard};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use crate::event_manager::EventManager;
use crate::keyboard_state::{KeyState, PressedKey};
use crate::mouse_state::MouseState;

pub struct Output {
	pix: Vec<Vec<bool>>,
	width: u32,
	height: u32,
	event_manager: Arc<Mutex<EventManager>>,
	sender: Sender<ScreenEvent>
}

enum ScreenEvent {
	Clear,
	Update,
	Buzz,
	StopBuzz,
	Set(u32, u32, bool)
}

fn get_sized_vec<T: Copy>(value: T, width: u32, height: u32) -> Vec<Vec<T>> {
	let mut new = Vec::new();
	for _ in 0..height {
		new.push(Vec::from([ value ].repeat(width as usize)));
	}
	new
}

struct SquareWave {
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

impl Output {
	pub fn new(width: u32, height: u32, scale: u32) -> Self {
		let (sender, receiver) = mpsc::channel();
		let event_manager = Arc::new(Mutex::new(EventManager::new()));
		let e = event_manager.clone();

		let s = sender.clone();
		thread::spawn(move || {
			loop {
				thread::sleep(Duration::from_millis(1000 / 60));
				s.send(ScreenEvent::Update)
					.unwrap_or_else(| e |
						log::error!("Error while sending screen update: {}", e)
					);
			}
		});

		thread::spawn(move || {
			let sdl_context = sdl2::init().unwrap();
			let mut canvas = sdl_context.video().unwrap()
				.window("CHIP 8", width * scale, height * scale)
				.position_centered()
				.build()
				.unwrap()
				.into_canvas().build().unwrap();

			let device = sdl_context.audio().unwrap()
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
				).unwrap();


			canvas.set_logical_size(width, height)
				.unwrap_or_else(| e |
					log::error!("Error while setting logical screen size (width: {}, height: {}): {}", width, height, e)
				);
			canvas.set_scale(scale as f32, scale as f32)
				.unwrap_or_else(| e |
					log::error!("Error while setting screen scale to {}: {}", scale, e)
				);

			let mut event_pump = sdl_context.event_pump().unwrap();
			loop {
				while let Ok(screen_event) = receiver.try_recv() {
					match screen_event {
						ScreenEvent::Update => canvas.present(),
						ScreenEvent::Buzz => device.resume(),
						ScreenEvent::StopBuzz => device.pause(),
						ScreenEvent::Clear => {
							canvas.set_draw_color(Color::BLACK);
							canvas.clear();
						}, ScreenEvent::Set(x, y, v) => {
							if v {
								canvas.set_draw_color(Color::WHITE)
							} else {
								canvas.set_draw_color(Color::BLACK);
							}
							canvas.draw_point(Point::new(x as i32, y as i32))
								.unwrap_or_else(| e |
									log::error!("Error while drawing a point at (x: {}, y: {}): {}", x, y, e)
								);
						}
					}
				}
				e.lock().unwrap().update(event_pump.poll_iter());
			}
		});

		let mut new = Self {
			pix: get_sized_vec(false, width, height),
			width,
			height,
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
		self.sender.send(ScreenEvent::Clear)
			.unwrap_or_else(| e |
				log::error!("Error while sending screen clear: {}", e)
			);
	}

	pub fn in_bounds(&self, x: u32, y: u32) -> bool {
		y < self.height && x < self.width
	}

	pub fn get(&self, x: u32, y: u32) -> bool {
		self.in_bounds(x, y) && self.pix[y as usize][x as usize]
	}

	pub fn set(&mut self, x: u32, y: u32, v: bool) {
		if self.in_bounds(x, y) {
			if self.pix[y as usize][x as usize] != v {
				self.pix[y as usize][x as usize] = v;
				self.sender.send(ScreenEvent::Set(x, y, v))
					.unwrap_or_else(| e |
						log::error!("Error while sending screen set_pix: {}", e)
					);
			}
		}
	}

	pub fn swap(&mut self, x: u32, y: u32) {
		self.set(x, y, !self.get(x, y));
	}

	pub fn is_pressed(&mut self, key: u8) -> bool {
		if let Some(k) = PressedKey::from_hex(key) {
			self.get_event_manager().is_key_pressed(k.into())
		} else {
			false
		}
	}

	pub fn buzz(&self) {
		self.sender.send(ScreenEvent::Buzz)
			.unwrap_or_else(| e |
				log::error!("Error while sending buzz: {}", e)
			);
	}

	pub fn stop_buzz(&self) {
		self.sender.send(ScreenEvent::StopBuzz)
			.unwrap_or_else(| e |
				log::error!("Error while sending buzz-stop: {}", e)
			);
	}

	pub fn get_current_input(&mut self) -> Option<u8> {
		self.get_event_manager()
			.get_pressed_keys()
			.iter()
			.filter_map(| k | k.to_hex())
			.next()
	}

	pub fn get_keyboard_state(&self) -> KeyState {
		KeyState::new(self.get_event_manager_pointer())
	}

	pub fn get_mouse_state(&self) -> MouseState {
		MouseState::new(self.get_event_manager_pointer())
	}
}
use std::sync::{Arc, mpsc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::{FullscreenType, WindowPos};
use crate::event_manager::EventManager;
use crate::app_state::{AppState, PressedKey};
use crate::audio_manager::AudioManager;
use crate::logger::LogError;


pub enum ScreenEvent {
	Clear,
	Update,
	Buzz,
	StopBuzz,
	ToggleFullscreen,
	RedrawAll(Vec<Vec<bool>>),
	Set(u32, u32, bool)
}

fn get_sized_vec<T: Copy>(value: T, width: u32, height: u32) -> Vec<Vec<T>> {
	let mut new = Vec::new();
	for _ in 0..height {
		new.push([ value ].repeat(width as usize));
	}
	new
}

pub struct Output {
	pix: Vec<Vec<bool>>,
	width: u32,
	height: u32,
	event_manager: Arc<Mutex<EventManager>>,
	sender: Sender<ScreenEvent>,
	size_changed: Arc<AtomicBool>
}

impl Output {
	pub fn new(width: u32, height: u32, scale: u32) -> Self {
		let (sender, receiver) = mpsc::channel();
		let size_changed = Arc::new(AtomicBool::new(false));
		let event_manager = Arc::new(Mutex::new(EventManager::new()));
		let e = event_manager.clone();
		let sc = size_changed.clone();

		thread::spawn(move || {
			let sdl_context = sdl2::init().unwrap();
			let video = sdl_context.video().unwrap();
			let mut window = video
				.window("CHIP 8", width * scale, height * scale)
				.resizable()
				.build()
				.unwrap();
			window.set_maximum_size(width * 200, height * 200)
				.elog("setting maximum (resizable) size");
			window.set_minimum_size(width, height)
				.elog("setting minimum (resizable) size");
			let mut size_before_fullscreen = window.size();
			let mut position_before_fullscreen = window.position();
			window.set_fullscreen(FullscreenType::Desktop)
				.elog("setting fullscreen @ beginning");

			let mut canvas = window.into_canvas().build().unwrap();

			let device = AudioManager::get_prepared_buzz_device(&sdl_context);


			let mut scale_x = scale;
			let mut scale_y = scale;

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
								canvas.set_draw_color(Color::WHITE);
							} else {
								canvas.set_draw_color(Color::BLACK);
							}
							canvas.fill_rect(Rect::new(
								x as i32 * scale_x as i32,
								y as i32 * scale_y as i32,
								scale_x,
								scale_y
							)).elog("drawing pixel");
						}, ScreenEvent::RedrawAll(pix) => {
							canvas.set_draw_color(Color::BLACK);
							canvas.clear();
							canvas.set_draw_color(Color::WHITE);
							canvas.fill_rects(&pix.iter().enumerate()
								.flat_map(| (y, r) |
									r.iter().enumerate()
										.filter(| (_, p) | **p)
										.map(move | (x, _) | Rect::new(
											(x as u32 * scale_x) as i32,
											(y as u32 * scale_y) as i32,
											scale_x,
											scale_y
										))
								).collect::<Vec<Rect>>()[..]).elog("redrawing all");
						}, ScreenEvent::ToggleFullscreen => {
							if canvas.window().fullscreen_state() == FullscreenType::Desktop {
								canvas.window_mut()
									.set_fullscreen(FullscreenType::Off)
									.elog("quitting fullscreen");
								canvas.window_mut()
									.set_size(
										size_before_fullscreen.0,
										size_before_fullscreen.1
									).elog("setting size after quitting fullscreen");
								canvas.window_mut()
									.set_position(
										WindowPos::Positioned(position_before_fullscreen.0),
										WindowPos::Positioned(position_before_fullscreen.1)
									);
							} else {
								size_before_fullscreen = canvas.window().size();
								position_before_fullscreen = canvas.window().position();
								canvas.window_mut()
									.set_fullscreen(FullscreenType::Desktop)
									.elog("entering fullscreen");
							}
						}
					}
				}
				for event in event_pump.poll_iter() {
					match event {
						Event::Window { win_event: WindowEvent::Resized(w, h), .. } => {
							canvas.window_mut()
								.set_size(w as u32, h as u32)
								.elog("setting window size after resize");
						}, Event::Window { win_event: WindowEvent::SizeChanged(w, h), .. } => {
							sc.store(true, Ordering::Relaxed);
							scale_x = w as u32 / width;
							scale_y = h as u32 / height;
						}, _ => e.lock().unwrap().update(event, (scale_x, scale_y))
					}
				}
			}
		});

		let mut new = Self {
			pix: get_sized_vec(false, width, height),
			width,
			height,
			event_manager,
			sender,
			size_changed
		};
		new.clear();
		new
	}

	pub fn update_screen(&self) {
		self.send(ScreenEvent::Update, "screen update");
	}

	pub fn toggle_fullscreen(&self) {
		self.send(ScreenEvent::ToggleFullscreen, "toggle fullscreen");
	}

	/// returns size changed & sets it to false
	pub fn size_changed(&mut self) -> bool {
		let b = self.size_changed.load(Ordering::Relaxed);
		self.size_changed.store(false, Ordering::Relaxed);
		b
	}

	pub fn get_event_manager(&self) -> MutexGuard<EventManager> {
		self.event_manager.lock().unwrap()
	}

	pub fn get_event_manager_pointer(&self) -> Arc<Mutex<EventManager>> {
		self.event_manager.clone()
	}

	pub fn clear(&mut self) {
		self.pix = get_sized_vec(false, self.width, self.height);
		self.send(ScreenEvent::Clear, "clear screen");
	}

	pub fn send(&self, s: ScreenEvent, msg: &str) {
		self.sender.send(s).elog(&format!("sending {}", msg));
	}

	pub fn in_bounds(&self, x: u32, y: u32) -> bool {
		y < self.height && x < self.width
	}

	pub fn get(&self, x: u32, y: u32) -> bool {
		self.in_bounds(x, y) && self.pix[y as usize][x as usize]
	}

	pub fn redraw_screen(&self) {
		self.send(ScreenEvent::RedrawAll(self.pix.clone()), "redraw all");
	}

	pub fn set(&mut self, x: u32, y: u32, v: bool) {
		if self.in_bounds(x, y) && self.pix[y as usize][x as usize] != v {
			self.pix[y as usize][x as usize] = v;
			self.send(ScreenEvent::Set(x, y, v), "set pix");
		}
	}

	/// returns true if a pix switched from `on` -> `off`
	pub fn swap(&mut self, x: u32, y: u32) -> bool {
		let old_val = self.get(x, y);
		self.set(x, y, !old_val);
		old_val
	}

	pub fn is_pressed(&self, key: u8) -> bool {
		if let Some(k) = PressedKey::from_hex(key) {
			self.get_event_manager().is_key_pressed(k)
		} else {
			false
		}
	}

	pub fn buzz(&self) {
		self.send(ScreenEvent::Buzz, "buzz");
	}

	pub fn stop_buzz(&self) {
		self.send(ScreenEvent::StopBuzz, "stop-buzz")
	}

	pub fn get_current_input(&mut self) -> Option<u8> {
		self.get_event_manager()
			.get_pressed_keys()
			.iter()
			.filter_map(| k | k.to_hex())
			.next()
	}

	pub fn get_input_state(&self) -> AppState {
		AppState::new(self.get_event_manager_pointer())
	}
}
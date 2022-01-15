use std::collections::HashMap;
use std::thread;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;
use sdl2::video::{FullscreenType, WindowPos};

use crate::events::app::AppEvent;
use crate::{LogError, PAUSE_TRANSPARENT_COLOR};
use crate::sdl2_interaction::audio_manager::{AudioEvent, AudioManager};
use crate::sdl2_interaction::event_manager::{AppEventManager, AppEventReceiver, AppEventSender, Event, IncomingEvent};
use crate::sdl2_interaction::screen::{Chip8BoolToColor, Chip8ColorToBool, Screen};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScreenEvent {
	Clear,
	Update,
	ToggleFullscreen,
	RedrawAll(HashMap<usize, HashMap<usize, Color>>),
	DrawPixel(usize, usize, Color),
	ScrollDown(isize),
	ScrollSide(isize),
	Any
}
impl Event for ScreenEvent {
	fn is_any(&self) -> bool {
		matches!(&self, &ScreenEvent::Any)
	}
}

pub struct Output {
	app_event_sender: AppEventSender,
	screen: Screen
}

impl Output {
	pub fn new(
		width: u32,
		height: u32,
		scale: u32,
		callback_receiver: AppEventReceiver,
		audio_callback_receiver: AppEventReceiver,
		app_event_sender: AppEventSender
	) -> Self {
		let mut event_manager = AppEventManager::new(app_event_sender.clone());

		thread::spawn(move || {
			let sdl_context = sdl2::init().unwrap();
			let video = sdl_context.video().unwrap();
			let mut window = video
				.window("CHIP 8", width * scale, height * scale)
				.resizable()
				.build()
				.unwrap();
			window.set_minimum_size(width, height)
				.elog("setting minimum (resizable) size");
			let mut size_before_fullscreen = window.size();
			let mut position_before_fullscreen = window.position();

			let mut canvas = window.into_canvas().build().unwrap();
			canvas.set_blend_mode(BlendMode::Blend);

			let mut audio_device = AudioManager::new(&sdl_context, audio_callback_receiver);

			let mut scale_x = scale;
			let mut scale_y = scale;

			let mut current_viewport = Rect::new(0, 0, width, height);

			let mut pause_overlay = false;

			let mut event_pump = sdl_context.event_pump().unwrap();
			loop {
				audio_device.update();
				while let Ok(app_event) = callback_receiver.try_recv() {
					match app_event {
						IncomingEvent::Pause(true) if !pause_overlay => {
							pause_overlay = true;
							canvas.set_draw_color(PAUSE_TRANSPARENT_COLOR);
							canvas.fill_rect(Rect::new(
								current_viewport.x() * scale_x as i32,
								current_viewport.y() * scale_y as i32,
								current_viewport.width() * scale_x,
								current_viewport.height() * scale_y,
							)).elog("making transparent pause overlay");
						}, IncomingEvent::Screen(s) => {
							match s {
								ScreenEvent::ScrollDown(s) => {
									current_viewport.y += s as i32;
									canvas.set_viewport(Rect::new(
										current_viewport.x() * scale_x as i32,
										current_viewport.y() * scale_y as i32,
										current_viewport.width() * scale_x,
										current_viewport.height() * scale_y,
									))
								},
								ScreenEvent::ScrollSide(s) => {
									current_viewport.x += s as i32;
									canvas.set_viewport(Rect::new(
										current_viewport.x() * scale_x as i32,
										current_viewport.y() * scale_y as i32,
										current_viewport.width() * scale_x,
										current_viewport.height() * scale_y,
									))
								}, ScreenEvent::DrawPixel(x, y, c) => {
									canvas.set_draw_color(c);
									canvas.fill_rect(Rect::new(
										x as i32 * scale_x as i32,
										y as i32 * scale_y as i32,
										scale_x,
										scale_y
									)).elog("drawing pixel");
								}, ScreenEvent::RedrawAll(pix) => {
									pause_overlay = false;
									canvas.set_draw_color(Color::BLACK);
									canvas.clear();
									canvas.set_draw_color(Color::WHITE);
									canvas.fill_rects(&pix.iter()
										.flat_map(|(y, r)|
											r.iter()
												.filter(|(_, p)| **p == Color::WHITE)
												.map(move |(x, _)| Rect::new(
													(*x as u32 * scale_x) as i32,
													(*y as u32 * scale_y) as i32,
													scale_x,
													scale_y
												))
										).collect::<Vec<Rect>>()[..]).elog("redrawing all");
								}, ScreenEvent::Update => canvas.present(),
								ScreenEvent::Clear => {
									canvas.set_draw_color(Color::BLACK);
									canvas.clear();
									canvas.present();
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
								}, _ => {}
							}
						}, IncomingEvent::App(AppEvent::WindowSizeChange(w, h)) => {
							scale_x = w as u32 / width;
							scale_y = h as u32 / height;
						}
						_ => {}
					}
				}
				for event in event_pump.poll_iter() {
					event_manager.update(event, (scale_x, scale_y));
				}
			}
		});

		Self {
			app_event_sender,
			screen: Screen::new()
		}
	}

	pub fn clear(&mut self) {
		self.screen.clear();
		self.send_to_app_state(ScreenEvent::Clear);
	}

	pub fn scroll_side(&mut self, amount: isize) {
		self.send_to_app_state(ScreenEvent::ScrollSide(amount));
		self.screen.set_scroll_side((self.screen.get_scroll_side() as isize + amount) as usize);
	}

	pub fn scroll_down(&mut self, amount: isize) {
		self.send_to_app_state(ScreenEvent::ScrollDown(amount));
		self.screen.set_scroll_down((self.screen.get_scroll_down() as isize + amount) as usize);
	}

	pub fn send_to_app_state(&self, s: ScreenEvent) {
		self.app_event_sender.send(IncomingEvent::Screen(s)).elog("sending something");
	}

	pub fn redraw_all(&self) {
		self.send_to_app_state(ScreenEvent::RedrawAll(self.screen.get_pixels()))
	}

	pub fn set(&mut self, x: usize, y: usize, v: bool) {
		for (x, y) in self.screen.get_pix(x, y) {
			if self.screen.set(x, y, v.into_color()) {
				self.send_to_app_state(ScreenEvent::DrawPixel(x, y, v.into_color()));
			}
		}
	}

	pub fn get_screen(&self) -> &Screen {
		&self.screen
	}

	pub fn get_screen_mut(&mut self) -> &mut Screen {
		&mut self.screen
	}

	/// returns true if a pix switched from `on` -> `off`
	pub fn swap(&mut self, x: usize, y: usize) -> bool {
		let old_val = self.screen.get(x, y).into_bool();
		self.set(x, y, !old_val);
		old_val
	}

	pub fn buzz(&self) {
		self.app_event_sender.send(IncomingEvent::Audio(AudioEvent::Buzz(true))).elog("buzzing");
	}

	pub fn stop_buzz(&self) {
		self.app_event_sender.send(IncomingEvent::Audio(AudioEvent::Buzz(false))).elog("stop buzzing");
	}
}
use std::io::{Write, stdout, Read, Cursor};
use std::ops::Deref;
use std::rc::Rc;
use crossterm::{execute, queue, cursor, style::{Color, Print}, event, Result, terminal, style::{self, Stylize}, event::{poll, read, Event, KeyCode}, ExecutableCommand, QueueableCommand};
use std::time::Duration;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use crossterm::style::ResetColor;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

fn get_sized_vec<T: Copy>(value: T, width: usize, height: usize) -> Vec<Vec<T>> {
	let mut new = Vec::new();
	for _ in 0..height {
		new.push(Vec::from([ value ].repeat(width)));
	}
	new
}

pub struct Screen {
	pix: Vec<Vec<bool>>,
	width: usize,
	height: usize,
	keys: Arc<Mutex<Option<char>>>,
	about_to_exit:Arc<Mutex<bool>>
}

impl Screen {
	pub fn new(width: usize, height: usize) -> Self {
		//clear screen & hide cursor
		let mut stdout = stdout();
		queue!(
			stdout,
			EnterAlternateScreen,
			crossterm::style::SetBackgroundColor(Color::Black),
			cursor::Hide
		);
		crossterm::terminal::enable_raw_mode();
		stdout.flush();
		let mut m = Arc::new(Mutex::new(None));
		let mut e = Arc::new(Mutex::new(false));
		let mut new = Self {
			// one line for blocking key request, other line for BUZZ
			pix: get_sized_vec(false, width, height + 2),
			width,
			height,
			keys: Arc::clone(&m),
			about_to_exit: Arc::clone(&e)
		};
		new.clear();
		thread::spawn(move||
			loop {
				match read().ok() {
					Some(Event::Key(event)) => {
						match event.code {
							KeyCode::Char(c) => {
								*m.lock().unwrap() = Some(c);
								thread::sleep(Duration::from_millis(100));
								*m.lock().unwrap() = None;
							}, KeyCode::Esc => *e.lock().unwrap() = true,
							_ => {}
						}
					},
					_ => {}
				}
			}
		);
		new
	}

	pub fn requested_exit(&self) -> bool {
		self.about_to_exit.lock().unwrap().clone()
	}

	pub fn exit(&self) {
		crossterm::terminal::disable_raw_mode();
		execute!(
			&mut stdout(),
			ResetColor,
			cursor::Show,
			cursor::MoveTo(1, self.height as u16 + 3),
			Print("Press enter to exit ...")
		);
		std::io::stdin().read_line(&mut String::new()).unwrap();
		stdout().execute(terminal::LeaveAlternateScreen);
		std::process::exit(0);
	}

	pub fn clear(&mut self) {
		self.pix = get_sized_vec(false, self.width, self.height);
		let mut stdout = stdout();
		for y in 1..=(self.height + 2) {
			for x in 1..=self.width {
				queue!(
					stdout,
					cursor::MoveTo(x as u16, y as u16),
					style::PrintStyledContent(" ".on_black())
				);
			}
		}
		stdout.flush();
	}

	pub fn in_bounds(&self, x: usize, y: usize) -> bool {
		y < self.height && x < self.width
	}

	pub fn get(&self, x: usize, y: usize) -> bool {
		self.in_bounds(x, y) && self.pix[y][x]
	}

	pub fn set(&mut self, x: usize, y: usize, v: bool) {
		if self.in_bounds(x, y) {
			self.pix[y][x] = v;
			queue!(
				&mut stdout(),
				cursor::MoveTo(x as u16 + 1, y as u16 + 1),
				style::PrintStyledContent(if v { " ".on_white() } else { " ".on_black() })
			);
		}
	}

	pub fn swap(&mut self, x: usize, y: usize) {
		self.set(x, y, !self.get(x, y));
	}

	pub fn update(&self) {
		stdout().flush();
	}

	pub fn get_current_input(&self) -> Option<u8> {
		if let Ok(v) = u8::from_str_radix(&self.keys.lock().unwrap().unwrap_or(' ').to_string(), 16) {
			return Some(v);
		}
		None
	}

	pub fn buzz(&self) {
		queue!(
			&mut stdout(),
			cursor::MoveTo(1, self.height as u16 - 2),
			style::PrintStyledContent("BUZZ".white().on_black())
		);
	}

	pub fn get_blocking_input(&self) -> u8 {
		self.update();
		queue!(
			&mut stdout(),
			cursor::MoveTo(1, self.height as u16 - 1),
			style::PrintStyledContent("Press a key ...".white().on_black())
		);
		loop {
			if let Some(e) = self.get_current_input() {
				break e;
			}
		}
	}
}
use std::io::{Write, stdout};
use crossterm::{
	execute, queue,
	cursor,
	style::{Color, Print},
	event,
	terminal , style::{self, Stylize},
	event::{poll, read, Event}
};
use std::time::Duration;

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
	height: usize
}
impl Screen {
	pub fn new(width: usize, height: usize) -> Self {
		//clear screen & hide cursor
		crossterm::terminal::enable_raw_mode();
		execute!(
			&mut stdout(),
			cursor::Hide,
			terminal::Clear(terminal::ClearType::All),
			crossterm::style::SetBackgroundColor(Color::White)
		);
		// TODO: Exit command - one thread to get terminal stuff
		/*thread::spawn(|| 
			loop {
				if poll(Duration::from_millis(100)).ok().unwrap() {
					match read().ok() {
							Some(Event::Key(event)) => {
								if let event::KeyCode::Char(c) = event.code {
									if c.to_ascii_lowercase() == 'q' {
										execute!(&mut stdout(), cursor::Show);
										crossterm::terminal::disable_raw_mode();
									}
								}
							},
							_ => {}
					}
				}
			}
		);*/
		Self {
			// one line for blocking key request, other line for BUZZ
			pix: get_sized_vec(false, width, height + 2),
			width,
			height
		}
	}

	pub fn clear(&mut self) {
		self.pix = get_sized_vec(false, self.width, self.height);
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
		}
	}

	pub fn swap(&mut self, x: usize, y: usize) {
		self.set(x, y, !self.get(x, y));
	}

	pub fn update(&self) {
		let mut stdout = stdout();
		self.pix.iter().enumerate().for_each(| (y, row) |
			row.iter().enumerate().for_each(| (x, o) |{
				queue!(
					stdout,
					cursor::MoveTo(x as u16 + 1, y as u16),
					style::PrintStyledContent(if *o { "█".black() } else { " ".white() })
				);
			})
		);
		stdout.flush();
	}

	pub fn get_current_input(&self) -> Option<u8> {
		if poll(Duration::from_millis(50)).ok().unwrap() {
			match read().ok() {
					Some(Event::Key(event)) => {
						if let event::KeyCode::Char(c) = event.code {
							if let Ok(v) = u8::from_str_radix(&c.to_string(), 16) {
								Some(v)
							} else {
								None
							}
						} else {
							None
						}
					},
					_ => None
			}
		} else {
			None
		}
	}

	pub fn buzz(&self) {
		execute!(
			&mut stdout(),
			cursor::MoveTo(1, self.height as u16 - 2),
			style::PrintStyledContent("BUZZ".black())
		);
	}

	pub fn get_blocking_input(&self) -> u8 {
		self.update();
		execute!(
			&mut stdout(),
			cursor::MoveTo(1, self.height as u16 - 1),
			style::PrintStyledContent("Press a key ...".black())
		);
		loop {
			if let Some(e) = self.get_current_input() {
				break e;
			}
		}
	}
}
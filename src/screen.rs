use std::io;
use std::io::{Write, stdout};
use crossterm::{QueueableCommand, cursor};

use crossterm::{
	execute,
	style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
	ExecutableCommand, Result,
	event, queue,
	terminal , style::{self, Stylize}
};

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
		execute!(&mut stdout(), cursor::Hide, terminal::Clear(terminal::ClearType::All));
		Self {
			pix: get_sized_vec(false, width, height),
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
		//reset cursor-pos and draw
		let mut stdout = stdout();
		//stdout.execute(cursor::MoveTo(0, 0));
		self.pix.iter().enumerate()
			.for_each(| (y, row) |
				row.iter().enumerate().for_each(| (x, o) |{
					queue!(
						stdout,
						cursor::MoveTo(x as u16, y as u16),
						style::PrintStyledContent((if *o { "#" } else { " " }).white())
					);
				})
			);
		stdout.flush();
	}

	pub fn get_input(&self) -> u8 {
		self.update();
		let mut s;
		loop {
			s = String::new();
			execute!(
				&mut stdout(),
				cursor::MoveTo(0, self.height as u16),
				style::PrintStyledContent("Press a hex key ...".red()),
				cursor::MoveTo(0, self.height as u16 + 1),
				Print(" ".repeat(self.width)),
				cursor::MoveTo(0, self.height as u16 + 1),
			);
			io::stdin().read_line(&mut s).expect("Failed to read line");
			if let Ok(v) = u8::from_str_radix(&s[0..1], 16) {
				execute!(
					&mut stdout(),
					cursor::MoveTo(0, self.height as u16),
					Print(" ".repeat(self.width)),
					cursor::MoveTo(0, self.height as u16 + 1),
					Print(" ".repeat(self.width)),
				);
				break v;
			}
		}
	}
}
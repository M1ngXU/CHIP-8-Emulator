fn get_sized_vec<T: Copy>(value: T, width: usize, height: usize) -> Vec<Vec<T>> {
	let mut new = Vec::new();
	for y in 0..height {
		new.push(Vec::new());
		for _ in 0..width {
			new[y].push(value);
		}
	}
	new
}

pub struct Screen {
	pix: Vec<Vec<bool>>
}
impl Screen {
	pub fn new(width: usize, height: usize) -> Self {
		//clear screen & hide cursor
		//println!("\x1B[2J\x1b[?25l");
		Self {
			pix: get_sized_vec(false, width, height)
		}
	}

	pub fn clear(&mut self) {
		self.pix = get_sized_vec(false, self.pix[0].len(), self.pix.len());
	}

	pub fn get(&self, x: usize, y: usize) -> bool {
		self.pix[y][x]
	}

	pub fn set(&mut self, x: usize, y: usize, v: bool) {
		self.pix[y][x] = v;
	}

	pub fn swap(&mut self, x: usize, y: usize) {
		self.set(x, y, !self.get(x, y));
	}

	pub fn update(&self) {
		//reset cursor-pos and draw
		/*println!(
			"\x1B[1;1H{}",
			self.pix.iter()
				.map(| row |
					row.iter().map(| o | if *o { "█"} else { " " })
						.collect::<Vec<&str>>().join("").to_string()
				).collect::<Vec<String>>().join("\n")
		);*/
	}
}
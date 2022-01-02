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
		println!("\x1B[2J\x1b[?25l");
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
		y < self.height || x < self.width
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
		println!(
			"\x1B[1;1H{}",
			self.pix.iter()
				.map(| row |
					row.iter().map(| o | if *o { "█" } else { " " })
						.collect::<Vec<&str>>().join("").to_string()
				).collect::<Vec<String>>().join("\n")
		);
	}
}
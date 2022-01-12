use sdl2::pixels::Color;

pub trait Chip8BoolToColor {
    fn into_color(self) -> Color;
}
impl Chip8BoolToColor for bool {
    fn into_color(self) -> Color {
        if self {
            Color::WHITE
        } else {
            Color::BLACK
        }
    }
}
pub trait Chip8ColorToBool {
    fn into_bool(self) -> bool;
}
impl Chip8ColorToBool for Color {
    fn into_bool(self) -> bool {
        self == Color::WHITE
    }
}

pub struct Screen {
    pixels: Vec<Vec<Color>>,
    width: usize,
    height: usize
}
impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        let mut new = Self {
            pixels: Vec::new(),
            width,
            height
        };
        new.clear();
        new
    }

    pub fn get_pixels(&self) -> Vec<Vec<Color>> {
        self.pixels.clone()
    }

    pub fn clear(&mut self) {
        self.pixels = Vec::new(); 
        for _ in 0..self.height {
            self.pixels.push([ Color::BLACK ].repeat(self.width as usize).to_vec());
        }
    }
    
    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        self.in_bounds(x, y).then(|| self.pixels[y][x])
    }

    pub fn set(&mut self, x: usize, y: usize, c: Color) -> bool {
        if self.get(x, y).is_some() {
            self.pixels[y][x] = c;
            true
        } else {
            false
        }
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }
}
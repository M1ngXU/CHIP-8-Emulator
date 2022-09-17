use sdl2::pixels::Color;
use std::collections::HashMap;

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
    pixels: HashMap<usize, HashMap<usize, Color>>,
    default_color: Color,
    scale: usize,
    scroll_down: usize,
    scroll_side: usize,
}
impl Screen {
    pub fn new() -> Self {
        Self {
            pixels: HashMap::new(),
            default_color: false.into_color(),
            scale: 2,
            scroll_down: 0,
            scroll_side: 0,
        }
    }

    pub fn get_pixels(&self) -> HashMap<usize, HashMap<usize, Color>> {
        self.pixels.clone()
    }

    pub fn clear(&mut self) {
        self.pixels = HashMap::new();
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        if let Some(row) = self.pixels.get(&(y * self.scale)) {
            if let Some(pix) = row.get(&(x * self.scale)) {
                return *pix;
            }
        }
        self.default_color
    }

    pub fn set_scroll_side(&mut self, s: usize) {
        self.scroll_side = s;
    }

    pub fn get_scroll_side(&self) -> usize {
        self.scroll_side
    }

    pub fn set_scroll_down(&mut self, s: usize) {
        self.scroll_down = s;
    }

    pub fn get_scroll_down(&self) -> usize {
        self.scroll_down
    }

    pub fn get_scale(&self) -> usize {
        self.scale
    }

    pub fn set_scale(&mut self, s: usize) {
        self.scale = s;
    }

    pub fn get_pix(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        if self.scale == 2 {
            vec![
                (self.scale * x, self.scale * y),
                (self.scale * x + 1, self.scale * y),
                (self.scale * x, self.scale * y + 1),
                (self.scale * x + 1, self.scale * y + 1),
            ]
        } else {
            vec![(x, y)]
        }
    }

    pub fn set(&mut self, x: usize, y: usize, c: Color) -> bool {
        if self.pixels.get(&y).is_none() {
            self.pixels.insert(y, HashMap::new());
        }
        if self.pixels.get(&y).unwrap().get(&x).is_none() {
            self.pixels
                .get_mut(&y)
                .unwrap()
                .insert(x, self.default_color);
        }
        if self.pixels.get(&y).unwrap().get(&x).unwrap() == &c {
            false
        } else {
            *self.pixels.get_mut(&y).unwrap().get_mut(&x).unwrap() = c;
            true
        }
    }
}

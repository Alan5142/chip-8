pub const DEFAULT_FONT_START_ADDRESS: u16 = 0x50;

pub const DEFAULT_FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[derive(Debug)]
pub struct Display {
    pub memory: [u8; WIDTH * HEIGHT]
}

#[derive(PartialEq, PartialOrd)]
pub enum Pixel {
    Off = 0,
    On = 1,
}

impl From<u8> for Pixel {
    fn from(v: u8) -> Self {
        if v == 0 { Pixel::Off } else { Pixel::On }
    }
}

impl Display {
    pub fn new() -> Display {
        Display {
            memory: [0; WIDTH * HEIGHT]
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, new_pixel: Pixel) {
        self.memory[x + y * WIDTH] = new_pixel as u8;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.memory[x + y * WIDTH].into()
    }

    pub fn clear_screen(&mut self) {
        self.memory = [0; WIDTH * HEIGHT];
    }

    pub fn is_pixel_set(&self, x: usize, y: usize) -> bool {
        self.get_pixel(x, y) == Pixel::On
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let height = sprite.len();
        let mut sprite_collision = false;

        for i in 0..height {
            let pixel = sprite[i];
            for j in 0..8 {
                if pixel & (0x80 >> i) != 0 {
                    let pos_x = (x + j) % WIDTH;
                    let pos_y = (y + i) % HEIGHT;
                    let old_value = self.get_pixel(pos_x, pos_y) as u8;
                    if old_value == 1 {
                        sprite_collision = true;
                    }
                    self.set_pixel(pos_x, pos_y, (pixel ^ old_value).into());
                }
            }
        }
        sprite_collision
    }
}

#[cfg(test)]
mod tests {
    use crate::display::{Display, Pixel};

    #[test]
    fn set_pixel() {
        let mut display = Display::new();

        display.set_pixel(0, 0, Pixel::On);
    }
}
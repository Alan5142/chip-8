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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Display {
    memory: [u8; WIDTH * HEIGHT],
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Pixel {
    Off = 0,
    On = 1,
}

impl From<u8> for Pixel {
    fn from(v: u8) -> Self {
        if v == 0 {
            Pixel::Off
        } else {
            Pixel::On
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub fn new() -> Display {
        Display {
            memory: [0; WIDTH * HEIGHT],
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
        let mut collision = false;
        for (j, row) in sprite.iter().enumerate() {
            for i in 0..8 {
                let new_value = row >> (7 - i) & 0x01;
                if new_value == 1 {
                    let xi = (x + i) % WIDTH;
                    let yj = (y + j) % HEIGHT;
                    let old_value = self.get_pixel(xi, yj) as u8;
                    if old_value == 1 {
                        collision = true;
                    }
                    self.set_pixel(xi, yj, Pixel::from(new_value ^ old_value));
                }
            }
        }


        collision
    }

    pub fn get_video_mem(&self) -> &[u8; WIDTH * HEIGHT] {
        &self.memory
    }
}

#[cfg(test)]
mod tests {
    use crate::display::{Display, Pixel};

    #[test]
    fn clear() {
        let mut display = Display::new();

        display.set_pixel(1, 1, Pixel::On);
        assert_eq!(display.is_pixel_set(1, 1), true);
        display.clear_screen();

        assert_eq!(Pixel::Off, display.get_pixel(1, 1));
    }

    #[test]
    fn set_pixel() {
        let mut display = Display::new();

        display.set_pixel(0, 0, Pixel::On);
    }

    #[test]
    fn draw() {
        let mut display = Display::new();

        let sprite: [u8; 2] = [0b00110011, 0b11001010];

        display.draw(0, 0, &sprite);

        assert_eq!(Pixel::Off, display.get_pixel(0, 0));
        assert_eq!(Pixel::Off, display.get_pixel(1, 0));
        assert_eq!(Pixel::On, display.get_pixel(2, 0));
        assert_eq!(Pixel::On, display.get_pixel(3, 0));
        assert_eq!(Pixel::Off, display.get_pixel(4, 0));
        assert_eq!(Pixel::Off, display.get_pixel(5, 0));
        assert_eq!(Pixel::On, display.get_pixel(6, 0));
        assert_eq!(Pixel::On, display.get_pixel(7, 0));

        assert_eq!(Pixel::On, display.get_pixel(0, 1));
        assert_eq!(Pixel::On, display.get_pixel(1, 1));
        assert_eq!(Pixel::Off, display.get_pixel(2, 1));
        assert_eq!(Pixel::Off, display.get_pixel(3, 1));
        assert_eq!(Pixel::On, display.get_pixel(4, 1));
        assert_eq!(Pixel::Off, display.get_pixel(5, 1));
        assert_eq!(Pixel::On, display.get_pixel(6, 1));
        assert_eq!(Pixel::Off, display.get_pixel(7, 1));
    }

    #[test]
    fn draw_detects_collisions() {
        let mut display = Display::new();

        let mut sprite: [u8; 1] = [0b00110000];
        let mut collision = display.draw(0, 0, &sprite);
        assert_eq!(false, collision);

        sprite = [0b00000011];
        collision = display.draw(0, 0, &sprite);
        assert_eq!(false, collision);

        sprite = [0b00000001];
        collision = display.draw(0, 0, &sprite);
        assert_eq!(true, collision);
    }
}

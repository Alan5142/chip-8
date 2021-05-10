/// KeyPad
/// |
pub struct KeyPad {
    keypad: [bool; 16],
}

impl KeyPad {
    pub fn new() -> KeyPad {
        KeyPad {
            keypad: [false; 16],
        }
    }

    pub fn any_key_pressed(&self) -> Option<u8> {
        let mut pressed_key = None;
        for (i, key) in self.keypad.iter().enumerate() {
            if *key {
                pressed_key = Some(i as u8);
            }
        }
        pressed_key
    }

    pub fn on_key(&mut self, key: u8, status: bool) {
        self.keypad[key as usize] = status;
    }

    pub fn is_key_down(&self, index: usize) -> bool {
        self.keypad[index]
    }
}

#[cfg(test)]
mod tests {
    use crate::keypad::KeyPad;

    #[test]
    fn press_key() {
        let mut keypad = KeyPad::new();
        assert_eq!(keypad.is_key_down(0), false);
        keypad.on_key(0, true);
        assert_eq!(keypad.is_key_down(0), true);
    }

    #[test]
    fn wait_for_key() {
        let mut keypad = KeyPad::new();
        for _ in 0..3 {
            keypad.any_key_pressed();
        }
        keypad.on_key(5, true);
        assert_eq!(keypad.any_key_pressed().unwrap(), 5);
    }
}
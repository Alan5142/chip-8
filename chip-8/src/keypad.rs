use std::ops::{Index, IndexMut};

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

    pub fn wait_for_key(&self) -> Option<u8> {
        let mut pressed_key = None;
        for (i, key) in self.keypad.iter().enumerate() {
            if *key {
                pressed_key = Some(i as u8);
            }
        }
        pressed_key
    }

    pub fn on_key_pressed(&mut self, key: u8, status: bool) {
        self.keypad[key as usize] = status;
    }

    pub fn is_key_down(&self, index: usize) -> bool {
        self.keypad[index]
    }
}

impl Index<u8> for KeyPad {
    type Output = bool;

    fn index(&self, index: u8) -> &Self::Output {
        &self.keypad[index as usize]
    }
}

impl IndexMut<u8> for KeyPad {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.keypad[index as usize]
    }
}

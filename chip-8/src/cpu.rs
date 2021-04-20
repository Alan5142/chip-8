use std::io::Read;

use rand::Rng;

use crate::display::{DEFAULT_FONTS, Display};
use crate::keypad::KeyPad;

pub struct Cpu {
    v: [u8; 16],
    memory: [u8; 4096],
    i: u16,
    stack: [u16; 24],
    program_counter: u16,
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: KeyPad,
    display: Display,
}

const START_ADDRESS: u16 = 0x200;

impl Cpu {
    fn execute_opcode(&mut self, opcode: u16) {
        let i_1 = (opcode & 0xF000) >> 12;
        let i_2 = (opcode & 0x0F00) >> 8;
        let i_3 = (opcode & 0x00F0) >> 4;
        let i_4 = opcode & 0x000F;

        self.program_counter += 2;

        let x = i_2 as u8;
        let y = i_3 as u8;
        let nn = (opcode & 0x00FF) as u8;

        match (i_1, i_2, i_3, i_4) {
            (0x0, 0x0, 0x0, 0x0) => self.program_counter -= 2,
            // Clear screen
            (0x0, 0x0, 0xE, 0x0) => self.display.clear_screen(),
            // Ret
            (0x0, 0x0, 0xE, 0xE) => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer as usize];
            }
            // Jump
            (0x1, _, _, _) => self.program_counter = opcode & 0x0FFF,
            // Call
            (0x2, _, _, _) => {
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = opcode & 0x0FFF;
            }
            // Skip if Vx = NN
            (0x3, _, _, _) => self.program_counter += if self.v[x as usize] == nn { 2 } else { 0 },

            // Skip if Vx != NN
            (0x4, _, _, _) => self.program_counter += if self.v[x as usize] != nn { 2 } else { 0 },
            // Skip if Vx == Vy
            (0x5, _, _, 0x0) => {
                self.program_counter += if self.v[x as usize] == self.v[y as usize] {
                    2
                } else {
                    0
                }
            }
            // Store NN in Vx
            (0x6, _, _, _) => self.v[x as usize] = nn,
            // Add nn to Vx
            (0x7, _, _, _) => {
                let (result, _) = self.v[x as usize].overflowing_add(nn);
                self.v[x as usize] = result;
            }
            // Set Vy = Vx
            (0x8, _, _, 0x0) => self.v[x as usize] = self.v[y as usize],
            // Set vX to Vx | Vy
            (0x8, _, _, 0x1) => self.v[x as usize] |= self.v[y as usize],
            // Set vX to Vx & Vy
            (0x8, _, _, 0x2) => self.v[x as usize] &= self.v[y as usize],
            // Set vX to Vx ^ Vy
            (0x8, _, _, 0x3) => self.v[x as usize] ^= self.v[y as usize],
            // Add Vx + Vy in Vx, set VF to 1 if overflow
            (0x8, _, _, 0x4) => {
                let (res, overflow) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[0x0F] = if overflow { 1 } else { 0 };
                self.v[x as usize] = res;
            }
            // Vx - Vy, set VF to 0 if borrow
            (0x8, _, _, 0x5) => {
                let (res, overflow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[0x0F] = if overflow { 0 } else { 1 };
                self.v[x as usize] = res;
            }
            // Vx = Vy >> 1, VF = LSB from Vy before op
            (0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[y as usize] & 0x1;
                self.v[x as usize] = self.v[y as usize] >> 1;
            }
            // Set Vx to Vy - Vx, VF=1 if borrow
            (0x8, _, _, 0x7) => {
                let (res, overflow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[0x0F] = if overflow { 0 } else { 1 };
                self.v[x as usize] = res;
            }
            // Vx = Vy << 1, VF = MSB from Vy before op
            (0x8, _, _, 0xE) => {
                self.v[0xF] = self.v[y as usize] & 0x80;
                self.v[x as usize] = self.v[y as usize] << 1;
            }
            // Skip instruction if Vx != Vy
            (0x9, _, _, 0x0) => {
                self.program_counter += if self.v[x as usize] != self.v[y as usize] {
                    2
                } else {
                    0
                }
            }
            // Store NNN in register I
            (0xA, _, _, _) => self.i = opcode & 0x0FFF,
            // Store NNN in register I
            (0xB, _, _, _) => self.program_counter = (opcode & 0x0FFF) + self.v[0] as u16,
            // Set Vx to random number with mask nn
            (0xC, _, _, _) => {
                self.v[x as usize] = rand::thread_rng().gen_range(0x0..0xFF) & nn;
            }
            // DRAW!!!
            (0xD, _, _, _) => {
                let n = opcode & 0x000F;
                let sprite_collision = self.display.draw(
                    self.v[x as usize] as usize,
                    self.v[y as usize] as usize,
                    &self.memory[self.i as usize..(self.i + n) as usize],
                );

                self.v[0xF] = if sprite_collision { 1 } else { 0 };
            }
            // Skip if key Vx is pressed
            (0xE, _, 0x9, 0xE) => {
                let vx = self.v[x as usize] as usize;
                self.program_counter += if self.keypad.is_key_down(vx) { 2 } else { 0 }
            }
            // Skip if key Vx is not pressed
            (0xE, _, 0xA, 0x1) => {
                let vx = self.v[x as usize] as usize;
                self.program_counter += if self.keypad.is_key_down(vx) { 0 } else { 2 }
            }
            // Set Vx value to delay timer
            (0xF, _, 0x0, 0x7) => self.v[x as usize] = self.delay_timer,
            // Wait for keypress
            (0xF, _, 0x0, 0xA) => {
                self.program_counter -= 2;
                if let Some(pressed) = self.keypad.wait_for_key() {
                    self.program_counter += 2;
                    self.v[x as usize] = pressed;
                }
            }
            // Set the delay timer to the value stored in Vx
            (0xF, _, 0x1, 0x5) => self.delay_timer = self.v[x as usize],
            // Set the sound timer to the value stored in Vx
            (0xF, _, 0x1, 0x8) => self.sound_timer = self.v[x as usize],
            // Set I = I + Vx
            (0xF, _, 0x1, 0xE) => self.i += self.v[x as usize] as u16,
            // Set I to the sprite stored in Vx
            (0xF, _, 0x2, 0x9) => self.i = self.v[x as usize] as u16 * 5,
            // Store the BCD of Vx in address I, I+1, I+2
            (0xF, _, 0x3, 0x3) => {
                let vx = self.v[x as usize];
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = (vx % 100) % 10;
            }
            // Set [I, I+X]
            // Store values V0 to Vx to address I to I + X, set I = I + X +1
            (0xF, _, 0x5, 0x5) => {
                self.memory[(self.i as usize)..(self.i as usize + x as usize + 1)]
                    .copy_from_slice(&self.v[0..(x + 1) as usize]);
                self.i += x as u16 + 1;
            }
            // Store values V0 to Vx to address I to I + X, set I = I + X +1
            (0xF, _, 0x6, 0x5) => {
                self.v[0..(x + 1) as usize].copy_from_slice(
                    &self.memory[(self.i as usize)..(self.i as usize + x as usize + 1)],
                );
                self.i += x as u16 + 1;
            }

            _ => {}
        }
    }

    pub fn new<Reader: Read>(mut file: Reader) -> std::io::Result<Box<Cpu>> {
        let mut cpu = Box::new(Cpu {
            v: [0; 16],
            memory: [0; 4096],
            i: 0,
            stack: [0; 24],
            program_counter: START_ADDRESS,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: KeyPad::new(),
            display: Display::new(),
        });
        for (i, item) in DEFAULT_FONTS.iter().enumerate() {
            cpu.memory[i] = *item;
        }

        let _ = file.read(&mut cpu.memory[(START_ADDRESS as usize)..])?;

        Ok(cpu)
    }

    pub fn get_next_instruction(&mut self) {}

    pub fn next(&mut self) {
        let instruction = (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize] as u16);

        self.execute_opcode(instruction);
    }

    pub fn set_key(&mut self, key_index: u8, status: bool) {
        self.keypad.on_key_pressed(key_index, status);
    }

    pub fn decrease_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn should_play_sound(&self) -> bool {
        self.sound_timer > 0
    }

    pub fn get_display(&self) -> &Display {
        &self.display
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::cpu::{Cpu, START_ADDRESS};
    use crate::display::Pixel;

    #[test]
    fn default_initialized() -> std::io::Result<()> {
        let data = [0; 0x200];
        let cpu = Cpu::new(Cursor::new(data))?;
        assert_eq!(&cpu.memory[0x200..0x400], &[0; 0x200]);
        assert_eq!(&cpu.v, &[0; 16]);
        assert_eq!(&cpu.stack, &[0; 24]);

        assert_eq!(cpu.sound_timer, 0);
        assert_eq!(cpu.delay_timer, 0);
        assert_eq!(cpu.i, 0);
        assert_eq!(cpu.stack_pointer, 0);
        assert_eq!(cpu.program_counter, START_ADDRESS);

        Ok(())
    }

    #[test]
    fn clear_display() -> std::io::Result<()> {
        let data = [0x00, 0xE0];
        let mut cpu = Cpu::new(Cursor::new(data))?;

        cpu.display.set_pixel(1, 1, Pixel::On);

        assert_eq!(Pixel::On, cpu.display.get_pixel(1, 1));

        cpu.next();
        assert_eq!(Pixel::Off, cpu.display.get_pixel(1, 1));

        Ok(())
    }

    #[test]
    fn test_jump() -> std::io::Result<()> {
        let instructions = [0x1F, 0xFF];
        let mut cpu = Cpu::new(Cursor::new(instructions))?;
        cpu.next();
        assert_eq!(cpu.program_counter, 0x0FFF, "El pc se actualizó");
        Ok(())
    }

    #[test]
    fn call_ret() -> std::io::Result<()> {
        let data = [0x2A, 0xBC];
        let mut cpu = Cpu::new(Cursor::new(data))?;

        cpu.memory[0x0ABC] = 0x00;
        cpu.memory[0x0ABD] = 0xEE;

        // call 0x0ABC
        cpu.next();
        // return
        cpu.next();

        assert_eq!(
            cpu.program_counter,
            START_ADDRESS + 0x02,
            "the program counter is updated to the new address"
        );
        assert_eq!(cpu.stack_pointer, 0, "the stack pointer is decremented");
        Ok(())
    }

    #[test]
    fn sne_vx_byte() -> std::io::Result<()> {
        let data = [0x45, 0x90, 0x45, 0x91];
        let mut cpu = Cpu::new(Cursor::new(data))?;
        cpu.v[5] = 0x90;

        cpu.next();

        // 0x90 == 0x90
        assert_eq!(cpu.program_counter, START_ADDRESS + 2, "not skips");

        cpu.next();

        // 0x90 != 0x91
        assert_eq!(cpu.program_counter, START_ADDRESS + 6, "skips");

        Ok(())
    }

    #[test]
    fn se_vx_byte() -> std::io::Result<()> {
        let data = [0x35, 0x90, 0x00, 0x00, 0x35, 0x91];
        let mut cpu = Cpu::new(Cursor::new(data))?;
        cpu.v[5] = 0x90;

        cpu.next();

        // 0x90 == 0x90
        assert_eq!(cpu.program_counter, START_ADDRESS + 4, "skips");

        cpu.next();

        // 0x90 != 0x91
        assert_eq!(cpu.program_counter, START_ADDRESS + 6, "not skips");

        Ok(())
    }

    #[test]
    fn se_vx_vy() -> std::io::Result<()> {
        let data = [0x55, 0x00, 0x55, 0x10];
        let mut cpu = Cpu::new(Cursor::new(data))?;
        cpu.v[5] = 0x90;
        cpu.v[0] = 0x91;
        cpu.v[1] = 0x90;

        cpu.next();

        // 0x90 != 0x91
        assert_eq!(cpu.program_counter, START_ADDRESS + 2, "not skips");

        cpu.next();

        // 0x90 == 0x90
        assert_eq!(cpu.program_counter, START_ADDRESS + 6, "skips");

        Ok(())
    }

    #[test]
    fn ld_vx_byte() -> std::io::Result<()> {
        let data = [0x60, 0xAA, 0x80, 0x10];
        let mut cpu = Cpu::new(Cursor::new(data))?;
        cpu.next();
        assert_eq!(cpu.v[0], 0xAA);
        cpu.next();
        assert_eq!(cpu.v[0], 0x00);

        Ok(())
    }

    #[test]
    fn add_vx_byte() -> std::io::Result<()> {
        let data = [0x75, 0x01];
        let mut cpu = Cpu::new(Cursor::new(data))?;
        cpu.v[5] = 3;

        cpu.next();

        assert_eq!(cpu.v[5], 4, "Se incrementó en 1");

        Ok(())
    }

    #[test]
    fn opcode_axxx() {
        let data = [0xAF, 0xAF];
        let mut cpu = Cpu::new(Cursor::new(data)).unwrap();
        cpu.next();

        assert_eq!(cpu.i, 0x0FAF, "the 'i' register is updated");
        assert_eq!(
            cpu.program_counter, 0x202,
            "the program counter is advanced two bytes"
        );
    }

    #[test]
    fn opcode_ld_i_addr() {
        let data = [0x61, 0xAA, 0x62, 0x1A, 0x6A, 0x15];

        let mut cpu = Cpu::new(Cursor::new(data)).unwrap();

        cpu.next();
        assert_eq!(cpu.v[1], 0xAA, "V1 is set");
        assert_eq!(
            cpu.program_counter, 0x202,
            "the program counter is advanced two bytes"
        );

        cpu.next();
        assert_eq!(cpu.v[2], 0x1A, "V2 is set");
        assert_eq!(
            cpu.program_counter, 0x204,
            "the program counter is advanced two bytes"
        );

        cpu.next();
        assert_eq!(cpu.v[10], 0x15, "V10 is set");
        assert_eq!(
            cpu.program_counter, 0x206,
            "the program counter is advanced two bytes"
        );
    }
}

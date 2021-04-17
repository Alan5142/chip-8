use std::io::Read;

use rand::Rng;

#[derive(Debug)]
pub struct Cpu {
    v: [u8; 16],
    memory: [u8; 4096],
    i: u16,
    stack: [u16; 24],
    program_counter: u16,
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [u32; 64 * 32],
    rng: rand::prelude::ThreadRng,
}

const START_ADDRESS: u16 = 0x200;
const DEFAULT_FONT_START_ADDRESS: u16 = 0x50;

const DEFAULT_FONTS: [u8; 80] = [
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

impl Default for Cpu {
    fn default() -> Self {
        let mut cpu = Cpu {
            v: [0; 16],
            memory: [0; 4096],
            i: 0,
            stack: [0; 24],
            program_counter: START_ADDRESS,
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [0; 64 * 32],
            rng: rand::thread_rng(),
        };
        cpu.memory[(DEFAULT_FONT_START_ADDRESS as usize)..80].copy_from_slice(&DEFAULT_FONTS);
        cpu
    }
}

impl Cpu {
    fn execute_opcode(&mut self, opcode: u16) {
        let i_1 = (opcode & 0xF000) >> 12;
        let i_2 = (opcode & 0x0F00) >> 8;
        let i_3 = (opcode & 0x00F0) >> 4;
        let i_4 = opcode & 0x000F;

        self.program_counter += 2;

        let x = self.v[i_2 as usize];
        let y = self.v[i_3 as usize];
        let nn = (opcode & 0x00FF) as u8;

        match (i_1, i_2, i_3, i_4) {
            // Clear screen
            (0x0, 0x0, 0xE, 0x0) => self.video = [0; 64 * 32],
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
                self.program_counter = opcode & 0x0FFF;
                self.stack_pointer += 1;
            }
            // Skip if Vx = NN
            (0x3, _, _, _) => self.program_counter += if self.v[x as usize] == nn { 2 } else { 0 },

            // Skip if Vx != NN
            (0x4, _, _, _) => self.program_counter += if self.v[x as usize] != nn { 2 } else { 0 },
            // Skip if Vx == Vy
            (0x5, _, _, 0x0) => self.program_counter += if self.v[x as usize] == self.v[y as usize] { 2 } else { 0 },
            // Store NN in Vx
            (0x6, _, _, _) => self.v[x as usize] = nn,
            // Add nn to Vx
            (0x7, _, _, _) => self.v[x as usize] += nn,
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
            (0x9, _, _, 0x0) => self.program_counter += if self.v[x as usize] != self.v[y as usize] { 2 } else { 1 },
            // Store NNN in register I
            (0xA, _, _, _) => self.i = opcode & 0x0FFF,
            // Store NNN in register I
            (0xB, _, _, _) => self.program_counter = (opcode & 0x0FFF) + self.v[0] as u16,
            // Set Vx to random number with mask nn
            (0xC, _, _, _) => self.v[x as usize] = self.rng.gen_range(0x0..0xFF) & nn,
            // ToDo: draw sprite at position Vx, Vy with N bytes starting at address I, Set VF to if changes
            (0xD, _, _, _) => {}
            // Skip if key Vx is pressed
            (0xE, _, 0x9, 0xE) => self.program_counter += if self.keypad[x as usize] == 1 { 2 } else { 0 },
            // Skip if key Vx is not pressed
            (0xE, _, 0xA, 0x1) => self.program_counter += if self.keypad[x as usize] == 1 { 0 } else { 2 },
            // Set Vx value to delay timer
            (0xF, _, 0x0, 0x6) => self.v[x as usize] = self.delay_timer,
            // Wait for keypress
            (0xF, _, 0x0, 0xA) => {
                self.program_counter -= 2;
                for (i, key) in self.keypad.iter().enumerate() {
                    if *key == 1 {
                        self.program_counter += 2;
                        self.v[x as usize] = i as u8;
                    }
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
                self.memory[self.i as usize + 2] = (vx / 100) % 10;
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
                self.v[0..(x + 1) as usize]
                    .copy_from_slice(&self.memory[(self.i as usize)..(self.i as usize + x as usize + 1)]);
                self.i += x as u16 + 1;
            }

            _ => {}
        }
    }

    pub fn new(file: &std::path::Path) -> std::io::Result<Box<Cpu>> {
        let mut file = std::fs::File::open(file)?;

        let mut cpu = Box::new(Cpu {
            ..Default::default()
        });

        file.read(&mut cpu.memory[(START_ADDRESS as usize)..])?;

        Ok(cpu)
    }

    pub fn get_next_instruction(&mut self) {}

    pub fn next(&mut self) {
        let instruction = (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize] as u16);

        self.program_counter += 2;
        self.execute_opcode(instruction);
    }
}

use raylib::prelude::*;
use std::{
    env,
    error::Error,
    fs::File,
    io::{Cursor, Read},
    process,
};

const FONTSET: [u8; 80] = [
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

struct Chip8 {
    memory: [u8; 4096],
    pc: u16,
    // I'm not sure what to do with the following commented fields:
    v: [u8; 16],
    i: u16,
    display: [[bool; 64]; 32],
    stack: [u16; 16],
    sp: usize,
    // delay_timer: u8,
    // sound_timer: u8,
}

impl Chip8 {
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.display[y][x]
    }

    pub fn flip_pixel(&mut self, x: usize, y: usize) -> bool {
        let was_on = self.display[y][x];
        self.display[y][x] = !self.display[y][x];
        was_on // return true if there was a collision
    }

    pub fn read_sprite_bytes(&self, start_addr: u16, count: u16) -> Vec<u8> {
        (0..count)
            .map(|i| self.memory[(start_addr + i) as usize])
            .collect()
    }

    /// For now, opens the rom file, and loads it into memory, and that's it
    pub fn new(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let mut chip = Chip8 {
            memory: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: [0; 16],
            sp: 0,
            display: [[false; 64]; 32],
            v: [0; 16],
        };

        // Loading font into memory
        let mut reader = Cursor::new(FONTSET);
        reader.read(&mut chip.memory[0x50..])?;

        // Opening file
        let mut file = File::open(&filepath)?;

        // Reading into "memory"
        // File::read(&mut file, &mut chip.memory[512..])?;
        File::read(&mut file, &mut chip.memory[0x200..])?;

        Ok(chip)
    }

    // Runs the main interpreter loop
    pub fn step(&mut self) -> Result<(), Box<dyn Error>> {
        // let mut opcode: [u8; 2] = [0; 2];
        // let mut reader = Cursor::new(&self.memory);
        // reader.set_position(self.pc as u64);

        // Fetch instruction directly from memory array
        if self.pc as usize + 1 >= self.memory.len() {
            return Err("PC out of bounds".into());
        }

        // fetch instruction
        let opcode = ((self.memory[self.pc as usize] as u16) << 8)
            | (self.memory[self.pc as usize + 1] as u16);

        // // Increment the PC by 2
        self.pc += 2;

        // Decode the instruction
        // First, do a match on full opcodes
        match opcode {
            0x00E0 => {
                println!("{:04X}: Clear screen", opcode);
            }
            0x00EE => {
                println!("{:04X}: Return from subroutine", opcode);
                if self.sp == 0 {
                    panic!("Return with empty stack!");
                }
                self.sp -= 1;
                self.pc = self.stack[self.sp];
                println!("Return to {:03X}", self.pc);
            }
            _ => (),
        };

        // Then, mask-based matches
        match opcode & 0xF000 {
            0x1000 => {
                // 1NNN - Jump to address NNN
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
                println!("pc is now: {}", nnn);
            }
            0x6000 => {
                // 6XNN - Set Vx = NN
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = nn;
                println!("{:03X}: Register{x} set to NN ({nn})", opcode);
                // println!("x: {}, nn: {}", x, nn);
            }
            0x7000 => {
                // 7XNN - Add value to register VX
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] += nn;
                println!("Added {nn} to Register{x}");
            }
            0xA000 => {
                let nnn = (opcode & 0x0FFF) as u16;
                self.i = nnn;
                println!("Set index register I to adress {:03X}", nnn);
            }
            0x2000 => {
                let nnn = opcode & 0x0FFF;
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = nnn;
                println!("Call subroutine at {:03X}", nnn);
            }
            0xD000 => {
                // DXYN
                // println!("{:04X}: Possible draw?", opcode);
                let vx = (opcode & 0x0F00) >> 8;
                let x = self.v[vx as usize] % 64;
                let vy = (opcode & 0x00F0) >> 4;
                let y = self.v[vy as usize] % 32;
                let n = opcode & 0x000F;

                self.v[0xF] = 0; // Clear collision flag
                println!(
                    "{:04X}: Draw an {} pixels tall sprite of {},{} from memory location {}",
                    opcode, n, x, y, self.i
                );

                let sprite_data = self.read_sprite_bytes(self.i, n);

                for (byte_index, &sprite_byte) in sprite_data.iter().enumerate() {
                    for i in (0..8).rev() {
                        let bit = (sprite_byte >> i) & 1;
                        let px = x + (7 - i);
                        let py = y as u16 + byte_index as u16;
                        if bit == 1 {
                            if self.flip_pixel(px as usize, py as usize) {
                                self.v[0xF] = 1;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        // println!("PC: {:03X}, Opcode: {:04X}", self.pc, opcode);

        Ok(())
    }

    // Just a test function to see if we loaded contents of the rom into "memory"
    // pub fn print_memory(&self) {
    //     for (index, chunk) in self.memory.chunks(16).enumerate() {
    //         print!("{:04X}: ", index * 16);
    //         for byte in chunk {
    //             print!("{:02X} ", byte);
    //         }
    //         println!()
    //     }
    //     println!("pc: {:X}, so: {:X}", self.pc, self.memory[self.pc]);
    // }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (mut rl, thread) = raylib::init().size(64 * 10, 32 * 10).title("Chip8").build();

    let filepath = &args[1];

    let mut chip8_game = match Chip8::new(&filepath) {
        Ok(game) => game,
        Err(err) => {
            eprintln!("Error starting Chip8 interpreter: {}", err);
            process::exit(1);
        }
    };

    rl.set_target_fps(60);
    while !rl.window_should_close() {
        // emulate e.g. 10 instructions per frame
        for _ in 0..10 {
            chip8_game.step().unwrap();
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Draw the display buffer
        for y in 0..32 {
            for x in 0..64 {
                if chip8_game.get_pixel(x, y) {
                    d.draw_rectangle(x as i32 * 10, y as i32 * 10, 10, 10, Color::WHITE);
                }
            }
        }
    }

    // chip8_game.print_memory();
    // chip8_game.run().unwrap();
}

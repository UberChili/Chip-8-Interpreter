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
    // V: [u8; 16],
    // i: u16,
    // stack: [u16; 16],
    // sp: u8,
    // delay_timer: u8,
    // sound_timer: u8,
}

impl Chip8 {
    /// For now, opens the rom file, and loads it into memory, and that's it
    pub fn new(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let mut chip = Chip8 {
            memory: [0; 4096],
            pc: 0x200,
            // V: [0; 16],
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
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut opcode: [u8; 2] = [0; 2];
        let mut reader = Cursor::new(&self.memory);
        reader.set_position(self.pc as u64);
        loop {
            // fetch instruction
            if let Err(_) = reader.read_exact(&mut opcode) {
                break;
            };
            let opcode = ((opcode[0] as u16) << 8) | opcode[1] as u16;
            self.pc += 2;

            // Decode the instruction
            // First full opcodes
            match opcode {
                0x00E0 => println!("Clear screen"),
                0x00EE => println!("Return from subroutine"),
                _ => (),
            };

            // Then mask-based matches
            match opcode & 0xF000 {
                0x6000 => {
                    // 6XNN - Set Vx = NN
                    println!("{:03X}", opcode);
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let nn = (opcode & 0x00FF) as u8;
                    println!("x: {}, nn: {}", x, nn);
                }
                0x1000 => {
                    // 1NNN - Jump to address ?
                    println!("{:03X}", opcode);
                    println!("1NNN, Jump to address");
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let nn = (opcode & 0x00FF) as u8;
                    println!("x: {}, nn: {}", x, nn);
                }
                _ => (),
            }
        }

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

    let filepath = &args[1];

    println!("We will attempt to work with the file {}", &filepath);

    let mut chip8_game = match Chip8::new(&filepath) {
        Ok(game) => game,
        Err(err) => {
            eprintln!("Error starting Chip8 interpreter: {}", err);
            process::exit(1);
        }
    };

    // chip8_game.print_memory();
    chip8_game.run().unwrap();
}

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
    pc: usize,
    // i: usize,
    // stack: [u8; 16],
    // stack: Vec<u8>,
    // delay_timer: u8,
    // sound_timer: u8,
    // V: [u8; 16],
}

impl Chip8 {
    /// For now, opens the rom file, and loads it into memory, and that's it
    pub fn new(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let mut chip = Chip8 {
            memory: [0; 4096],
            pc: 0x200,
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
            self.pc += 2;

            // test print instruction
            // let instruction: u16 = u16::from_be_bytes(opcode);
            // println!("Instruction at pc {:X}: {:X}", self.pc, instruction);
            // println!(
            //     "Instruction at pc {:X}: {:X} {:X}",
            //     self.pc, &opcode[0], &opcode[1]
            // );

            // Decode the instruction
            // Decode the instruction
            match opcode {
                [0x00, 0xE0] => println!("Clear screen!"),
                [0x00, 0xEE] => println!("Return from subroutine"),
                [first, second] if first & 0xF0 == 0x10 => {
                    // 1NNN - Jump to address
                    let addr = ((first as u16 & 0x0F) << 8) | second as u16;
                    println!("Jump to address {:03X}", addr);
                }
                [first, second] if first & 0xF0 == 0x60 => {
                    // 6XNN - Set VX = NN
                    let x = (first & 0x0F) as usize;
                    println!("Set V{:X} = {:02X}", x, second);
                }
                [first, second] if first & 0xF0 == 0xA0 => {
                    // ANNN - Set I = NNN
                    let addr = ((first as u16 & 0x0F) << 8) | second as u16;
                    println!("Set I = {:03X}", addr);
                }
                _ => println!("Unknown instruction: {:02X}{:02X}", opcode[0], opcode[1]),
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

use std::time::{Duration, Instant};
use std::env;
use std::fs;
use std::thread::sleep;

const RAM_SIZE: usize = 4096;
const NUM_REG: usize = 16;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_WIDTH: usize = 64;
//const FONT_SIZE: usize = 80;
//const STACK_SIZE: usize = 16;
const BYTE: usize = 8;
const FLAG_REGISTER: usize = 0xF;
const INSTRUCTIONS_PER_SECOND: u64 = 700;
const NANOS_PER_INSTRUCTION: u64 = 1_000_000_000/INSTRUCTIONS_PER_SECOND;

/**
 * fonts
 */
// const FONTS: [u8; FONT_SIZE] = [
//     0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
//     0x20, 0x60, 0x20, 0x20, 0x70, // 1
//     0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
//     0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
//     0x90, 0x90, 0xF0, 0x10, 0x10, // 4
//     0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
//     0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
//     0xF0, 0x10, 0x20, 0x40, 0x40, // 7
//     0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
//     0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
//     0xF0, 0x90, 0xF0, 0x90, 0x90, // A
//     0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
//     0xF0, 0x80, 0x80, 0x80, 0xF0, // C
//     0xE0, 0x90, 0x90, 0x90, 0xE0, // D
//     0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
//     0xF0, 0x80, 0xF0, 0x80, 0x80  // F 
// ]; 
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($arg)*);
    };
}
struct Chip8 {
    memory: [u8; RAM_SIZE],
    i_reg: u16, //holds the memory addr of sprite data
    pc: u16,
    //stack: [u16; STACK_SIZE],
    registers: [u8; NUM_REG],
    //delay_timer: u8,
    //sound_timer: u8,
    display: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
}

impl Chip8 {

    fn render(&mut self) {
        print!("\x1B[H");
        for row in (0..self.display.len()).step_by(SCREEN_WIDTH) {
            for pixel in row..row+SCREEN_WIDTH {
                if self.display[pixel] {
                    print!("█");
                } else {
                    print!(" ")
                }
            }
            println!();
        }
        
    }
    fn fetch(&mut self) -> u16 {
        let instr_hi = self.memory[self.pc as usize];
        let instr_lo = self.memory[self.pc as usize + 1];
        let instr = (instr_hi as u16) << BYTE | instr_lo as u16 ;
        //debug
        debug_println!("(pc, instr): ({:02X?},{:04X?})", self.pc, instr);
        self.pc += 2;

        instr
    }

    //decode happens here too
    fn execute(&mut self, instr: u16) {
        let nibble: u8 = ((instr & 0xFF00) >> 12) as u8;
        debug_println!("opcode: {:02X?}", nibble);

        match nibble {
           0x0 =>  {
            self.display = [false; SCREEN_HEIGHT * SCREEN_WIDTH]
           },
           0x1 => {
            let addr = instr & 0xFFF;
            self.pc = addr;
            debug_println!("pc = {:04X?}", self.pc);
            debug_println!("addr = {:04X?}", addr);
           },
           0x6 => {
            let reg = ((instr & 0xF00) >> BYTE) as usize;
            let val = (instr & 0xFF) as u8;
            self.registers[reg] = val;
            debug_println!("Register[{reg}]: {:02X?}", self.registers[reg]);
           },
           0x7 => {
            let reg = ((instr & 0xF00) >> BYTE) as usize;
            let val = (instr & 0xFF) as u8;
            self.registers[reg] += val;
           }
           0xA => {
            let addr = instr & 0xFFF;
            self.i_reg = addr;
            debug_println!("i_reg: {:04X?}", self.i_reg);
           }
           0xD => {
            let reg_x = ((instr & 0xF00) >> BYTE) as usize;
            let reg_y = ((instr & 0x0F0) >> 4) as usize;
            let x = self.registers[reg_x] as usize % SCREEN_WIDTH;
            let y = self.registers[reg_y] as usize % SCREEN_HEIGHT;
            let height = (instr & 0xF) as usize;
            
            self.registers[FLAG_REGISTER] = 0;

            debug_println!("register[{reg_x}]: {x}");
            debug_println!("register[{reg_y}]: {y}");
            debug_println!("height(hex,dec): ({:02X?}, {height})", {height});
            //set display
            for i in 0..height {
                if y + i <= SCREEN_HEIGHT {
                    let sprite_data = self.memory[(self.i_reg as usize) + i];
                    //for each bit in sprite_data
                    for bit in 0..BYTE {
                        if y + bit <= SCREEN_WIDTH {
                            let sprite_bit = (sprite_data >> ((BYTE - 1) - bit)) & 1;
                            let flatten_coord = ((y + i) * SCREEN_WIDTH) + (x + bit);
                            let screen_pixel = &mut self.display[flatten_coord];

                            if sprite_bit == 1 && *screen_pixel == true {
                                self.registers[FLAG_REGISTER] = 1;
                                *screen_pixel = false;
                            } else if sprite_bit == 1 && *screen_pixel == false {
                                *screen_pixel = true;
                            }
                        }   
                    }
                }
            }

           }
           _ => {
            println!("Not implemented yet!");
           }
        }

    }

    fn tick(&mut self) {
        //fetch
        let instr = self.fetch();
        //decode & execute
        self.execute(instr);
    }
}


    
fn main() -> Result<(), Box<dyn std::error::Error + 'static>> { 
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rom_path>", args[0]);
        std::process::exit(1);
    }

    //instantiating chip8
    let mut chip8 = Chip8 {
        memory: [0; RAM_SIZE],
        pc: 0x200,
        //stack: [0; STACK_SIZE],
        registers: [0; NUM_REG],
        i_reg: 0,
        //delay_timer: 0,
        //sound_timer: 0,
        display: [false; SCREEN_HEIGHT * SCREEN_WIDTH]        
    };

    //load rom
    let rom = fs::read(&args[1])?;
    chip8.memory[0x200..0x200 + rom.len()].copy_from_slice(&rom);
    
    //debug
    for (addr, data) in chip8.memory.chunks(16).enumerate() {
        debug_println!("{:04X}: {:02X?}", addr * 16, data);
    }
    
    
    let mut last_tick = Instant::now();
    print!("\x1B[H");
    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(last_tick);

        if elapsed >= Duration::from_nanos(NANOS_PER_INSTRUCTION) {
            chip8.tick();
            last_tick = now;
        }

        chip8.render();
        sleep(Duration::from_millis(16)); // ~60fps
    
    }

}
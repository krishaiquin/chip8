use std::time::{Duration, Instant};
use std::env;
use std::fs;
use std::thread::sleep;

const RAM_SIZE: usize = 4096;
const NUM_REG: usize = 16;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_WIDTH: usize = 64;
//const FONT_SIZE: usize = 80;
const STACK_SIZE: usize = 16; 
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
    stack: Vec<u16>,
    registers: [i8; NUM_REG],
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

    fn combine_u8(&self, hi: u8, lo: u8) -> i8 {
        ((hi << 4) | lo) as i8
    }
    fn combine_u16(&self, hi: u8, mid: u8, lo: u8) -> u16 {
        (hi << 8) as u16 | (mid << 4) as u16 | lo as u16
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
        let nibble1: u8 = ((instr & 0xF000) >> 12) as u8;
        let nibble2: u8 = ((instr & 0xF00) >> 8) as u8;
        let nibble3: u8 = ((instr & 0xF0) >> 4) as u8;
        let nibble4: u8 = (instr & 0xF)  as u8;
        debug_println!("instr: {:04X?}", instr);
        debug_println!("nibbles: {:02X?}, {:02X?}, {:02X?}, {:02X?}", nibble1, nibble2, nibble3, nibble4);

        match (nibble1, nibble2, nibble3, nibble4) {
            (0x0, 0x0, 0xE, 0x0) => {
                self.display = [false; SCREEN_HEIGHT * SCREEN_WIDTH];
                self.render();
            },
            (0x1, _, _, _) => {
                let addr = self.combine_u16(nibble2, nibble3, nibble4);
                self.pc = addr;
                debug_println!("(pc = addr): ({:04X?} = {:04X?})", self.pc, addr);
            },
            (0x2, _, _, _) => {
                let addr = self.combine_u16(nibble2, nibble3, nibble4);
                self.stack.push(self.pc);
                debug_println!("PC {:04X?} pushed to stack!", self.stack.last().expect("Expecting a value from stack but got None."));
                self.pc = addr;
                debug_println!("PC has been set to {:04X?}", self.pc);
            },
            (0x0, 0x0, 0xE, 0xE) => {
                debug_println!("Popping {:04X?} from stack.", self.stack.last());
                self.pc = self.stack.pop().expect("Expecting a value from stack but got None.");
                debug_println!("PC has been set to {:04X?}", self.pc);
            },
            (0x3, _, _, _) => {
                let regx = self.registers[nibble2 as usize];
                let value = self.combine_u8(nibble3, nibble4);
                debug_println!("(regx == value): ({regx} == {value})"); 
                if regx == value {
                    self.pc += 2;
                }
            },
            (0x4, _, _, _) => {
                let regx = self.registers[nibble2 as usize];
                let value = self.combine_u8(nibble3, nibble4);
                debug_println!("(regx != value): ({regx} != {value})"); 
                if regx != value {
                    self.pc += 2;
                }
            },
            (0x5, _, _, 0) => {
                let regx = self.registers[nibble2 as usize];
                let regy = self.registers[nibble3 as usize];
                debug_println!("(regx == regy): ({regx} == {regy})"); 
                if regx == regy {
                    self.pc += 2;
                }
            },
            (0x9, _, _, 0) => {
                let regx = self.registers[nibble2 as usize];
                let regy = self.registers[nibble3 as usize];
                debug_println!("(regx != regy): ({regx} != {regy})"); 
                if regx != regy {
                    self.pc += 2;
                }
            },
            (0x6, _, _, _) => {
                let value = self.combine_u8(nibble3, nibble4);
                self.registers[nibble2 as usize] = value;
            }, 
            (0x7, _, _, _) => {
                let value = self.combine_u8(nibble3, nibble4); 
                self.registers[nibble2 as usize] += value;
 
            },
            (0x8, _, _, 0) => { 
                self.registers[nibble2 as usize] = self.registers[nibble3 as usize];
            },
            (0x8, _, _, 1) => {
                self.registers[nibble2 as usize] = self.registers[nibble2 as usize] | self.registers[nibble3 as usize];
            },
            (0x8, _, _, 2) => {
                self.registers[nibble2 as usize] = self.registers[nibble2 as usize] & self.registers[nibble3 as usize];
            },
            (0x8, _, _, 3) => {
                self.registers[nibble2 as usize] = self.registers[nibble2 as usize] ^ self.registers[nibble3 as usize];
            },
            //
            // Overflow occurs when :
            // 1. Two negative numbers are added and an answer comes positive
            // 2. Two positive numbers are added and an answer comes negative
            //
            (0x8, _, _, 4) => {
                let regx = self.registers[nibble2 as usize];
                let regy = self.registers[nibble3 as usize]; 
                let result = regx + regy;
                self.registers[nibble2 as usize] = result;

                if (regx < 0 && regy < 0 && result > 0) || (regx > 0 && regy > 0 && result < 0) {
                    self.registers[FLAG_REGISTER] = 1;
                } else {
                    self.registers[FLAG_REGISTER] = 0;
                }
            },
            //
            // FLAG_REGISTER = 1 when:
            // 1. The first operand is larger than or equal to the second operand
            // FLAG_REGISETER = 0 when: (underflow)
            // 1. The second operand is larger than the first operand
            //
            (0x8, _, _, 5) => {
                let regx = self.registers[nibble2 as usize];
                let regy = self.registers[nibble3 as usize];
                self.registers[nibble2 as usize] = regx - regy;

                if regx >= regy {
                    self.registers[FLAG_REGISTER] = 1; 
                } else {
                    self.registers[FLAG_REGISTER] = 0; 
                }
            },
            (0x8, _, _, 7) => {
                let regx = self.registers[nibble2 as usize];
                let regy = self.registers[nibble3 as usize];
                self.registers[nibble2 as usize] = regy - regx;

                if regy >= regx {
                    self.registers[FLAG_REGISTER] = 1; 
                } else {
                    self.registers[FLAG_REGISTER] = 0; 
                }
            },
            (0x8, _, _, 6) => {
                let regx = self.registers[nibble2 as usize];
                let rightmost_bit = regx & 0x1;

                // Shift VX by 1 bit to the right
                self.registers[nibble2 as usize] = regx >> 1;
                // set VF if bit shift out was 1
                if rightmost_bit == 1 {
                    self.registers[FLAG_REGISTER] = 1; 
                } else {
                    self.registers[FLAG_REGISTER] = 0; 
                }
            },
            (0x8, _, _, 0xE) => {
                let regx = self.registers[nibble2 as usize];
                let leftmost_bit = regx & 0x10;

                // Shift VX by 1 bit to the right
                self.registers[nibble2 as usize] = regx << 1;
                // set VF if bit shift out was 1
                if leftmost_bit == 0x10 {
                    self.registers[FLAG_REGISTER] = 1; 
                } else {
                    self.registers[FLAG_REGISTER] = 0; 
                }
            },
            (_, _, _, _) => {
                println!("Invalid opcode");
            }
        //    0x0 =>  {
        //     self.display = [false; SCREEN_HEIGHT * SCREEN_WIDTH]
        //    },
        //    0x1 => {
        //     let addr = instr & 0xFFF;
        //     self.pc = addr;
        //     debug_println!("pc = {:04X?}", self.pc);
        //     debug_println!("addr = {:04X?}", addr);
        //    },
        //    0x6 => {
        //     let reg = ((instr & 0xF00) >> BYTE) as usize;
        //     let val = (instr & 0xFF) as u8;
        //     self.registers[reg] = val;
        //     debug_println!("Register[{reg}]: {:02X?}", self.registers[reg]);
        //    },
        //    0x7 => {
        //     let reg = ((instr & 0xF00) >> BYTE) as usize;
        //     let val = (instr & 0xFF) as u8;
        //     self.registers[reg] += val;
        //    }
        //    0xA => {
        //     let addr = instr & 0xFFF;
        //     self.i_reg = addr;
        //     debug_println!("i_reg: {:04X?}", self.i_reg);
        //    }
        //    0xD => {
        //     let reg_x = ((instr & 0xF00) >> BYTE) as usize;
        //     let reg_y = ((instr & 0x0F0) >> 4) as usize;
        //     let x = self.registers[reg_x] as usize % SCREEN_WIDTH;
        //     let y = self.registers[reg_y] as usize % SCREEN_HEIGHT;
        //     let height = (instr & 0xF) as usize;
            
        //     self.registers[FLAG_REGISTER] = 0;

        //     debug_println!("register[{reg_x}]: {x}");
        //     debug_println!("register[{reg_y}]: {y}");
        //     debug_println!("height(hex,dec): ({:02X?}, {height})", {height});
        //     //set display
        //     for i in 0..height {
        //         let row = y + i;
        //         if row <= SCREEN_HEIGHT {
        //             let sprite_data = self.memory[(self.i_reg as usize) + i];
        //             //for each bit in sprite_data
        //             for bit in 0..BYTE {
        //                 let col = x + bit;
        //                 if col <= SCREEN_WIDTH {
        //                     let sprite_bit = (sprite_data >> ((BYTE - 1) - bit)) & 1;
        //                     let flatten_coord = (row * SCREEN_WIDTH) + col;
        //                     let screen_pixel = &mut self.display[flatten_coord];

        //                     if sprite_bit == 1 && *screen_pixel == true {
        //                         self.registers[FLAG_REGISTER] = 1;
        //                         *screen_pixel = false;
        //                     } else if sprite_bit == 1 && *screen_pixel == false {
        //                         *screen_pixel = true;
        //                     }
        //                 }   
        //             }
        //         }
        //     }

        //    }
        //    _ => {
        //     println!("Not implemented yet!");
        //    }
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
        stack: Vec::with_capacity(STACK_SIZE),
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
pub const SCREEN_HEIGHT: usize = 64;
pub const SCREEN_WIDTH: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

pub struct Emu {
    pc: u16, // Program Counter - keeps the index of the current instruction
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,               // I reg used for indexing into RAM for read and writes
    stack: [u16; STACK_SIZE], // Stack is used when you are entering or exiting a subroutine. follows LIFO
    sp: u16,                  // Stack Pointer is used for indexing the stack
    keys: [bool; NUM_KEYS],
    dt: u8, // Delay timer counts down every cycle and performing some action if it hits 0
    st: u8, // Sound timer counts down every cycle and upon hitting 0 emits a noise
}

impl Emu {
    pub fn new() -> Self {
        Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        }
    }
}

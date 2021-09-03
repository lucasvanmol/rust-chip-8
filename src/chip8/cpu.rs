use crate::chip8::opcodes::*;
use crate::chip8::display::Display;
use crate::chip8::registers::{Registers, Register};
use rand::random;
use std::io;
use std::{fs::File, io::Read};
use either::Either;


const SPRITE_BYTE_LENGTH: usize = 5;
const SPRITES: [u8; SPRITE_BYTE_LENGTH * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xf0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

pub struct CHIP8 {
    stack: Vec<u16>,
    ram: [u8; 0xFFF],
    reg: Registers,
    display: Display,
}

impl CHIP8 {
    pub fn new() -> Self {
        let mut ram = [0; 0xFFF];
        ram[..80].clone_from_slice(&SPRITES);
        CHIP8 { 
            stack: Vec::with_capacity(16),
            ram: ram,
            reg: Registers::new(),
            display: Display::init(),
        }
    }

    fn decode_instruction(bytes: u16) -> Instruction {
        match get_first(bytes) {
            0x0 => {
                if bytes == 0x00E0 {
                    return Instruction::CLS;
                } else if bytes == 0x00EE {
                    return Instruction::RET;
                }           
                return Instruction::SYS(get_addr(bytes));
            }
            0x1 => { Instruction::JP(get_addr(bytes)) }
            0x2 => { Instruction::CALL(get_addr(bytes)) }
            0x3 => { Instruction::SE(get_vx(bytes), Either::Right(get_byte(bytes))) }
            0x4 => { Instruction::SNE(get_vx(bytes), Either::Right(get_byte(bytes))) }
            0x5 => { Instruction::SE(get_vx(bytes), Either::Left(get_vy(bytes))) }
            0x6 => { Instruction::LD(get_vx(bytes), Either::Right(get_byte(bytes))) }
            0x7 => { Instruction::ADD(get_vx(bytes), Either::Right(get_byte(bytes))) }
            0x8 => {
                match get_nibble(bytes) {
                    0x0 => { Instruction::LD(get_vx(bytes), Either::Left(get_vy(bytes))) }
                    0x1 => { Instruction::OR(get_vx(bytes), get_vy(bytes)) }
                    0x2 => { Instruction::AND(get_vx(bytes), get_vy(bytes)) }
                    0x3 => { Instruction::XOR(get_vx(bytes), get_vy(bytes)) }
                    0x4 => { Instruction::ADD(get_vx(bytes), Either::Left(get_vy(bytes))) }
                    0x5 => { Instruction::SUB(get_vx(bytes), get_vy(bytes)) }
                    0x6 => { Instruction::SHR(get_vx(bytes)) }
                    0x7 => { Instruction::SUBN(get_vx(bytes), get_vy(bytes)) }
                    0xE => { Instruction::SHL(get_vx(bytes)) }
                    _ => { panic!("Unrecognized OP Code 0x{:X}", bytes) }
                }
            }
            0x9 => { Instruction::SNE(get_vx(bytes), Either::Left(get_vy(bytes))) }
            0xA => { Instruction::LD_I(get_addr(bytes)) }
            0xB => { Instruction::JP_V0(get_addr(bytes)) }
            0xC => { Instruction::RND(get_vx(bytes), get_byte(bytes)) }
            0xD => { Instruction::DRW(get_vx(bytes), get_vy(bytes), get_nibble(bytes)) }
            0xE => {
                match bytes.to_be_bytes()[1] {
                    0x9E => { Instruction::SKP(get_vx(bytes)) }
                    0xA1 => { Instruction::SKNP(get_vx(bytes)) }
                    _ => { panic!("Unrecognized OP Code 0x{:X}", bytes) }
                }
            }
            0xF => {
                match bytes.to_be_bytes()[1] {
                    0x07 => { Instruction::LD_Vx_DT(get_vx(bytes)) }
                    0x0A => { Instruction::LD_Vx_K(get_vx(bytes)) }
                    0x15 => { Instruction::LD_DT_Vx(get_vx(bytes)) }
                    0x18 => { Instruction::LD_ST_Vx(get_vx(bytes)) }
                    0x1E => { Instruction::ADD_I(get_vx(bytes)) }
                    0x29 => { Instruction::LD_F(get_vx(bytes)) }
                    0x33 => { Instruction::LD_B(get_vx(bytes)) }
                    0x55 => { Instruction::LD_I_Vx(get_vx(bytes)) }
                    0x65 => { Instruction::LD_Vx_I(get_vx(bytes)) }
                    _ => { panic!("Unrecognized OP Code 0x{:X}", bytes) }
                }
            }
            _ => { unreachable!() }
        }
    }

    fn get_vx_val(&self, reg: &Register) -> Option<u8> {
        match reg {
            Register::Vx(num) => Some(self.reg.Vx[*num as usize])
        }
    }

    fn set_vx_val(&mut self, reg: &Register, val: u8) {
        match reg {
            Register::Vx(num) => self.reg.Vx[*num as usize] = val
        }
    }

    fn execute_instruction(&mut self, instr: Instruction) {
        match instr {
            Instruction::SYS(_) => {
                // ignored
            },
            Instruction::CLS => {
                self.display.clear();
                self.display.update_buffer();
            },
            Instruction::RET => {
                self.reg.PC = self.stack.pop().unwrap().clone() as usize; 
                self.reg.SP = self.reg.SP.wrapping_sub(1);
            },
            Instruction::JP(addr) => {
                self.reg.PC = addr as usize;
            },
            Instruction::JP_V0(addr) => {
                self.reg.PC = (addr + self.reg.Vx[0] as u16) as usize;
            },
            Instruction::CALL(addr) => {
                self.reg.SP += 1; 
                self.stack.push(self.reg.PC as u16); 
                self.reg.PC = addr as usize;
            },
            Instruction::SE(vx, other) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = match other {
                    Either::Left(reg) => self.get_vx_val(&reg).unwrap(),
                    Either::Right(u8) => u8
                };
                if val1 == val2 {
                    self.reg.PC += 2
                }
            },
            Instruction::SNE(vx, other) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = match other {
                    Either::Left(reg) => self.get_vx_val(&reg).unwrap(),
                    Either::Right(u8) => u8
                };
                if val1 != val2 {
                    self.reg.PC += 2
                }
            },
            Instruction::ADD(vx, other) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = match other {
                    Either::Left(reg) => self.get_vx_val(&reg).unwrap(),
                    Either::Right(u8) => u8
                };
                let result = val1.overflowing_add(val2);
                self.set_vx_val(&vx, result.0);
                self.set_vx_val(&Register::Vx(0xF), result.1 as u8);
            },
            Instruction::ADD_I(vx) => {
                self.reg.I += self.get_vx_val(&vx).unwrap() as u16
            },
            Instruction::SUB(vx, vy) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = self.get_vx_val(&vy).unwrap();
                let result = val1.overflowing_sub(val2);
                self.set_vx_val(&vx, result.0);
                self.set_vx_val(&Register::Vx(0xF), !result.1 as u8);
            },
            Instruction::SUBN(vx, vy) => {
                self.execute_instruction(Instruction::SUB(vy, vx))
            },
            Instruction::OR(vx, vy) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = self.get_vx_val(&vy).unwrap();
                self.set_vx_val(&vx, val1 | val2)
            },
            Instruction::AND(vx, vy) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = self.get_vx_val(&vy).unwrap();
                self.set_vx_val(&vx, val1 & val2)
            },
            Instruction::XOR(vx, vy) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                let val2 = self.get_vx_val(&vy).unwrap();
                self.set_vx_val(&vx, val1 ^ val2)
            },
            Instruction::SHR(vx) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                self.set_vx_val(&Register::Vx(0xF), (val1.trailing_ones() > 0) as u8);
                self.set_vx_val(&vx, val1 >> 1)
            },
            Instruction::SHL(vx) => {
                let val1 = self.get_vx_val(&vx).unwrap();
                self.set_vx_val(&Register::Vx(0xF), (val1.leading_ones() > 0) as u8);
                self.set_vx_val(&vx, val1 << 1)
            },
            Instruction::RND(vx, byte) => {
                let rand: u8 = random();
                self.set_vx_val(&vx, rand & byte);
            },
            Instruction::DRW(vx, vy, nibble) => {
                let start = self.reg.I as usize;
                let end = (self.reg.I + nibble as u16) as usize;
                let bytes = &self.ram[start .. end];
                let collision = self.display.set_pixels(
                    self.get_vx_val(&vx).unwrap(), 
                    self.get_vx_val(&vy).unwrap(), 
                    bytes
                );
                self.display.update_buffer();
                self.set_vx_val(&Register::Vx(0xF), collision as u8);
            },
            Instruction::SKP(vx) => {
                let val = self.get_vx_val(&vx).unwrap();
                let key = map_u8_to_key(val)
                    .expect(format!("Invalid key value {:?} in register {:?} used in SKP instruction", val, vx).as_ref());
                if self.display.is_key_down(key) {
                    self.reg.PC += 2;
                }
            },
            Instruction::SKNP(vx) => {
                let val = self.get_vx_val(&vx).unwrap();
                let key = map_u8_to_key(val)
                    .expect(format!("Invalid key value {:?} in register {:?} used in SKNP instruction", val, vx).as_ref());
                if !self.display.is_key_down(key) {
                    self.reg.PC += 2;
                }
            },
            Instruction::LD(vx, other) => {
                let val = match other {
                    Either::Left(reg) => self.get_vx_val(&reg).unwrap(),
                    Either::Right(u8) => u8
                };
                self.set_vx_val(&vx, val);
            },
            Instruction::LD_I(addr) => {
                self.reg.I = addr;
            }
            Instruction::LD_Vx_DT(vx) => {
                self.set_vx_val(&vx, self.reg.get_dt())
            },
            Instruction::LD_Vx_K(vx) => {
                while self.display.is_window_open() {
                    if let Some(key) = self.display.get_key_down() {
                        if let Some(val) = map_key_to_u8(key) {
                            self.set_vx_val(&vx, val);
                            break;
                        }
                    }
                }
            },
            Instruction::LD_DT_Vx(vx) => {
                self.reg.set_dt(self.get_vx_val(&vx).unwrap());
            },
            Instruction::LD_ST_Vx(vx) => {
                self.reg.set_st(self.get_vx_val(&vx).unwrap());
            },
            Instruction::LD_F(vx) => {
                let val =self.get_vx_val(&vx).unwrap();
                self.reg.I = CHIP8::get_sprite_addr(val)
                    .expect(format!("Tried to get sprite with hex {:X}", val).as_ref());
            },
            Instruction::LD_B(vx) => {
                let val = self.get_vx_val(&vx).unwrap();
                let bcd = to_bcd(val);
                self.ram[self.reg.I as usize] = bcd[0];
                self.ram[(self.reg.I + 1) as usize] = bcd[1];
                self.ram[(self.reg.I + 2) as usize] = bcd[2];
            },
            Instruction::LD_I_Vx(vx) => {
                match vx {
                    Register::Vx(byte) => {
                        for i in 0..byte+1 {
                            let vx = Register::Vx(i);
                            let val = self.get_vx_val(&vx).unwrap();
                            self.ram[(self.reg.I + i as u16) as usize] = val;
                        }
                    },
                }
            },
            Instruction::LD_Vx_I(vx) => {
                match vx {
                    Register::Vx(byte) => {
                        for i in 0..byte+1 {
                            let vx = Register::Vx(i);
                            let val = self.ram[(self.reg.I + i as u16) as usize];
                            self.set_vx_val(&vx, val)
                        }
                    },
                }
            },
        }
    }

    pub fn load(&mut self, filename: String) -> Result<(), io::Error> {
        let mut f = File::open(&filename)?;
        f.read(&mut self.ram[0x200..])?;
        Ok(())
    }

    pub fn run(&mut self) {
        while self.display.is_window_open() && self.reg.PC + 1 <= self.ram.len() {
            let opcode: u16 = self.ram[self.reg.PC] as u16 * 0x0100 + self.ram[self.reg.PC + 1] as u16;
            let instr = CHIP8::decode_instruction(opcode);
            let mut increment = true;
            match instr {
                Instruction::JP(_) | Instruction::JP_V0(_) | Instruction::CALL(_) => { increment = false }
                _ => {}
            }

            dbg!(&instr);

            self.execute_instruction(instr);
            
            // thread::sleep(Duration::from_nanos(1));
            if increment {
                self.reg.PC += 2;
            }
        }
    }

    fn get_sprite_addr(hex: u8) -> Option<u16> {
        if hex > 0xF {
            None
        } else {
            Some(hex as u16 * SPRITE_BYTE_LENGTH as u16)
        }
    }
}
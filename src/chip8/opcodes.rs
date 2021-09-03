use minifb::Key;
use either::Either;
use crate::chip8::registers::Register;

pub type Address = u16;
pub type Nibble = u8;
pub type OPcode = u16;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Instruction {
    SYS(Address), // Ignored?
    CLS,
    RET,
    JP(Address),
    JP_V0(Address),
    CALL(Address),
    SE(Register, Either<Register, u8>),
    SNE(Register, Either<Register, u8>),
    ADD(Register, Either<Register, u8>),
    ADD_I(Register),
    SUB(Register, Register),
    SUBN(Register, Register),
    OR(Register, Register),
    AND(Register, Register),
    XOR(Register, Register),
    SHR(Register),
    SHL(Register),
    RND(Register, u8),
    DRW(Register, Register, Nibble),
    SKP(Register),
    SKNP(Register),
    LD(Register, Either<Register, u8>),
    LD_I(Address),
    LD_Vx_DT(Register),
    LD_Vx_K(Register),
    LD_DT_Vx(Register),
    LD_ST_Vx(Register),
    LD_F(Register),
    LD_B(Register),
    LD_I_Vx(Register),
    LD_Vx_I(Register),
}

pub fn get_first(bytes: OPcode) -> u8 {
    (bytes >> 12) as u8
}

pub fn get_addr(bytes: OPcode) -> Address {
    bytes & 0x0FFF
}

pub fn get_vx(bytes: OPcode) -> Register {
    Register::Vx(((bytes & 0x0F00) >> 8) as u8)
}

pub fn get_vy(bytes: OPcode) -> Register {
    Register::Vx(((bytes & 0x00F0) >> 4) as u8)
}

pub fn get_nibble(bytes: OPcode) -> Nibble {
    (bytes & 0x000F) as u8
}

pub fn get_byte(bytes: OPcode) -> u8 {
    (bytes & 0x00FF) as u8
}

pub fn map_key_to_u8(key: Key) -> Option<u8> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xC),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None
    }
}

pub fn map_u8_to_key(val: u8) -> Option<Key> {
    match val {
        0x1 => Some(Key::Key1),
        0x2 => Some(Key::Key2),
        0x3 => Some(Key::Key3),
        0xC => Some(Key::Key4),
        0x4 => Some(Key::Q),
        0x5 => Some(Key::W),
        0x6 => Some(Key::E),
        0xD => Some(Key::R),
        0x7 => Some(Key::A),
        0x8 => Some(Key::S),
        0x9 => Some(Key::D),
        0xE => Some(Key::F),
        0xA => Some(Key::Z),
        0x0 => Some(Key::X),
        0xB => Some(Key::C),
        0xF => Some(Key::V),
        _ => None
    }
}

pub fn to_bcd(byte: u8) -> [u8; 3] {
    let ones = byte % 10;
    let tens = (byte % 100) / 10;
    let hundreds = (byte - ones - tens) / 100;
    [hundreds, tens, ones]
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_opcodes() {
        const TESTCODE: u16 = 0x1234;
        
        assert_eq!(get_first(TESTCODE), 0x1);
        assert_eq!(get_addr(TESTCODE), 0x0234);
        assert_eq!(get_vx(TESTCODE), Register::Vx(0x2));
        assert_eq!(get_vy(TESTCODE), Register::Vx(0x3));
        assert_eq!(get_nibble(TESTCODE), 0x4);
        assert_eq!(get_byte(TESTCODE), 0x34)
    }

    #[test]
    fn test_bcd() {
        assert_eq!(to_bcd(255), [2,5,5]);
        assert_eq!(to_bcd(12), [0,1,2]);
        assert_eq!(to_bcd(8), [0,0,8]);
        assert_eq!(to_bcd(0), [0,0,0]);
    }
}
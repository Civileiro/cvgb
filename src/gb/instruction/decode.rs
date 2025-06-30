use modular_bitfield::prelude::*;
use thiserror::Error;

use crate::gb::core::Core;

use super::{Cond, Instruction, R8, R16, R16mem, R16stk};

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("invalid opcode {0:?}")]
    InvalidOpcode(u8),
    #[error("not enough bytes to decode")]
    OutOfBytes,
}

impl Instruction {
    pub fn decode(core: &Core) -> Result<Self, DecodeError> {
        let pc = core.get_pc();
        let bytes = [core.read(pc), core.read(pc + 1), core.read(pc + 2)];
        Self::decode_next(bytes.into_iter())
    }
    pub fn decode_next<I: Iterator<Item = u8>>(mut bytes: I) -> Result<Self, DecodeError> {
        let next = |b: &mut I| b.next().ok_or(DecodeError::OutOfBytes);
        let opcode = next(&mut bytes)?;
        let imm8 = next;
        let imm16 = |b: &mut I| Ok(u16::from_le_bytes([next(b)?, next(b)?]));
        let r16 = || R16::from_bytes((opcode & 0b00110000) >> 4).unwrap();
        let r16mem = || R16mem::from_bytes((opcode & 0b00110000) >> 4).unwrap();
        let r16stk = || R16stk::from_bytes((opcode & 0b00110000) >> 4).unwrap();
        let r8l = || R8::from_bytes((opcode & 0b00111000) >> 3).unwrap();
        let r8r = || R8::from_bytes(opcode & 0b00000111).unwrap();
        let cond = || Cond::from_bytes((opcode & 0b00011000) >> 3).unwrap();
        let instruction = match opcode & 0xC0 {
            // Block 0
            0x00 => match opcode & 0x0F {
                0x00 | 0x08 => match opcode >> 3 {
                    0b000 => Self::NOP,
                    0b001 => Self::STRsp {
                        imm16_mem: imm16(&mut bytes)?,
                    },
                    0b010 => Self::STOP,
                    0b011 => Self::JR {
                        imm8: imm8(&mut bytes)?,
                    },
                    _ => Self::JRcond {
                        imm8: imm8(&mut bytes)?,
                        cond: cond(),
                    },
                },
                0x01 => Self::LDRimm {
                    dest: r16(),
                    imm16: imm16(&mut bytes)?,
                },
                0x02 => Self::STRmem { r16_mem: r16mem() },
                0x03 => Self::INCr16 { r16: r16() },
                0x04 | 0x0C => Self::INCr8 { r8: r8l() },
                0x05 | 0x0D => Self::DECr8 { r8: r8l() },
                0x06 | 0x0E => Self::LDRr8 {
                    r8: r8l(),
                    imm8: imm8(&mut bytes)?,
                },
                0x07 | 0x0F => match opcode >> 3 {
                    0b000 => Self::RLCA,
                    0b001 => Self::RRCA,
                    0b010 => Self::RLA,
                    0b011 => Self::RRA,
                    0b100 => Self::DAA,
                    0b101 => Self::CPL,
                    0b110 => Self::SCF,
                    0b111 => Self::CCF,
                    _ => unreachable!(),
                },
                0x09 => Self::ADDhl { r16: r16() },
                0x0A => Self::LDRmem { r16_mem: r16mem() },
                0x0B => Self::DECr16 { r16: r16() },
                _ => unreachable!(),
            },
            // Block 1
            0x40 => match opcode {
                0b01110110 => Self::HALT,
                _ => Self::LDR {
                    dest: r8l(),
                    src: r8r(),
                },
            },
            // Block 2
            0x80 => match (opcode & 0b00111000) >> 3 {
                0 => Self::ADD { r8: r8r() },
                1 => Self::ADC { r8: r8r() },
                2 => Self::SUB { r8: r8r() },
                3 => Self::SBC { r8: r8r() },
                4 => Self::AND { r8: r8r() },
                5 => Self::XOR { r8: r8r() },
                6 => Self::OR { r8: r8r() },
                7 => Self::CP { r8: r8r() },
                _ => unreachable!(),
            },
            // Block 3
            0xC0 => match opcode & 0x0F {
                0x01 => Self::POP { r16stk: r16stk() },
                0x05 => Self::PUSH { r16stk: r16stk() },
                0x06 | 0x0E => match (opcode & 0b00111000) >> 3 {
                    0b000 => Self::ADDimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b001 => Self::ADCimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b010 => Self::SUBimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b011 => Self::SBCimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b100 => Self::ANDimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b101 => Self::XORimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b110 => Self::ORimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b111 => Self::CPimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    _ => unreachable!(),
                },
                0x07 | 0x0F => Self::RST {
                    tgt3: (opcode & 0b00111000) >> 3,
                },
                _ => match opcode & 0b00111111 {
                    0b000000 | 0b001000 | 0b010000 | 0b011000 => Self::RETcond { cond: cond() },
                    0b001001 => Self::RET,
                    0b011001 => Self::RETI,
                    0b000010 | 0b001010 | 0b010010 | 0b011010 => Self::JPcond {
                        cond: cond(),
                        imm16: imm16(&mut bytes)?,
                    },
                    0b000011 => Self::JPimm {
                        imm16: imm16(&mut bytes)?,
                    },
                    0b101001 => Self::JPhl,
                    0b000100 | 0b001100 | 0b010100 | 0b011100 => Self::CALLcond {
                        cond: cond(),
                        imm16: imm16(&mut bytes)?,
                    },
                    0b001101 => Self::CALLimm {
                        imm16: imm16(&mut bytes)?,
                    },
                    0b001011 => {
                        let cb_opcode = imm8(&mut bytes)?;
                        let upper = cb_opcode >> 6;
                        let r8 = R8::from_bytes(cb_opcode & 0b00000111).unwrap();
                        let b3 = (cb_opcode >> 3) & 0b00000111;
                        match (upper, b3) {
                            (0, 0) => Self::RLC { r8 },
                            (0, 1) => Self::RRC { r8 },
                            (0, 2) => Self::RL { r8 },
                            (0, 3) => Self::RR { r8 },
                            (0, 4) => Self::SLA { r8 },
                            (0, 5) => Self::SRA { r8 },
                            (0, 6) => Self::SWAP { r8 },
                            (0, 7) => Self::SRL { r8 },
                            (1, b3) => Self::BIT { b3, r8 },
                            (2, b3) => Self::RES { b3, r8 },
                            (3, b3) => Self::SET { b3, r8 },
                            _ => unreachable!(),
                        }
                    }
                    0b100010 => Self::STH,
                    0b100000 => Self::STHaddr {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b101010 => Self::STRaddr {
                        imm16: imm16(&mut bytes)?,
                    },
                    0b110010 => Self::LDH,
                    0b110000 => Self::LDHaddr {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b111010 => Self::LDaddr {
                        imm16: imm16(&mut bytes)?,
                    },
                    0b101000 => Self::ADDimm {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b111000 => Self::LDhl {
                        imm8: imm8(&mut bytes)?,
                    },
                    0b111001 => Self::LDsp,
                    0b110011 => Self::DI,
                    0b111011 => Self::EI,
                    _ => return Err(DecodeError::InvalidOpcode(opcode)),
                },
            },
            _ => unreachable!(),
        };
        Ok(instruction)
    }
}

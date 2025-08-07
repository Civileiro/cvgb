use std::fmt::{Display, Write};

use modular_bitfield::prelude::*;

use super::{
    Cpu, CpuContext,
    instructions::{InputU8, InputU16, OutputU8, OutputU16},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    pub a: u8,
    pub f: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    #[skip]
    __: B4,
    pub c: bool,
    pub h: bool,
    pub n: bool,
    pub z: bool,
}

impl Registers {
    pub fn inc_pc(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }
    pub fn dec_pc(&mut self) {
        self.pc = self.pc.wrapping_sub(1);
    }
    pub fn get_c_flag(&self) -> bool {
        self.f.c()
    }
    pub fn set_c_flag(&mut self, val: bool) {
        self.f.set_c(val);
    }
    pub fn get_h_flag(&self) -> bool {
        self.f.h()
    }
    pub fn set_h_flag(&mut self, val: bool) {
        self.f.set_h(val);
    }
    pub fn get_n_flag(&self) -> bool {
        self.f.n()
    }
    pub fn set_n_flag(&mut self, val: bool) {
        self.f.set_n(val);
    }
    pub fn get_z_flag(&self) -> bool {
        self.f.z()
    }
    pub fn set_z_flag(&mut self, val: bool) {
        self.f.set_z(val);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl Display for Reg16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Reg16::*;
        match self {
            AF => f.write_str("af"),
            BC => f.write_str("bc"),
            DE => f.write_str("de"),
            HL => f.write_str("hl"),
            SP => f.write_str("sp"),
        }
    }
}

impl InputU16<Reg16> for Cpu {
    fn read16(&mut self, _: &mut impl CpuContext, reg: Reg16) -> u16 {
        self.regs.get16(reg)
    }
}
impl OutputU16<Reg16> for Cpu {
    fn write16(&mut self, _: &mut impl CpuContext, reg: Reg16, data: u16) {
        self.regs.set16(reg, data)
    }
}

impl Registers {
    pub fn get16(&self, reg: Reg16) -> u16 {
        use Reg16::*;
        match reg {
            AF => <u16>::from_be_bytes([self.a, self.f.into()]),
            BC => <u16>::from_be_bytes([self.b, self.c]),
            DE => <u16>::from_be_bytes([self.d, self.e]),
            HL => <u16>::from_be_bytes([self.h, self.l]),
            SP => self.sp,
        }
    }
    pub fn set16(&mut self, reg: Reg16, val: u16) {
        use Reg16::*;
        match reg {
            AF => {
                let [a, f] = val.to_be_bytes();
                self.a = a;
                self.f = f.into();
            }
            BC => {
                let [b, c] = val.to_be_bytes();
                self.b = b;
                self.c = c;
            }
            DE => {
                let [d, e] = val.to_be_bytes();
                self.d = d;
                self.e = e;
            }
            HL => {
                let [h, l] = val.to_be_bytes();
                self.h = h;
                self.l = l;
            }
            SP => self.sp = val,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Display for Reg8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Reg8::*;
        match self {
            A => f.write_char('a'),
            B => f.write_char('b'),
            C => f.write_char('c'),
            D => f.write_char('d'),
            E => f.write_char('e'),
            H => f.write_char('h'),
            L => f.write_char('l'),
        }
    }
}

impl InputU8<Reg8> for Cpu {
    fn read(&mut self, _: &mut impl CpuContext, reg: Reg8) -> u8 {
        self.regs.get8(reg)
    }
}
impl OutputU8<Reg8> for Cpu {
    fn write(&mut self, _: &mut impl CpuContext, reg: Reg8, data: u8) {
        self.regs.set8(reg, data);
    }
}

impl Registers {
    pub fn get8(&self, reg: Reg8) -> u8 {
        use Reg8::*;
        match reg {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }
    pub fn set8(&mut self, reg: Reg8, val: u8) {
        use Reg8::*;
        match reg {
            A => self.a = val,
            B => self.b = val,
            C => self.c = val,
            D => self.d = val,
            E => self.e = val,
            H => self.h = val,
            L => self.l = val,
        }
    }
}

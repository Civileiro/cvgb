use std::fmt::Display;

use crate::gb::core::Core;
use enum_assoc::Assoc;
use modular_bitfield::prelude::*;

#[derive(Debug, Specifier, Assoc)]
#[func(pub fn mneumonic(&self) -> &str)]
#[func(pub fn get(&self, core: &Core) -> u8)]
#[func(pub fn set(&self, core: &mut Core, data: u8))]
pub enum R8 {
    #[assoc(mneumonic = "b", get = core.get_b(), set = core.set_b(data))]
    B = 0b000,
    #[assoc(mneumonic = "c", get = core.get_c(), set = core.set_c(data))]
    C = 0b001,
    #[assoc(mneumonic = "d", get = core.get_d(), set = core.set_d(data))]
    D = 0b010,
    #[assoc(mneumonic = "e", get = core.get_e(), set = core.set_e(data))]
    E = 0b011,
    #[assoc(mneumonic = "h", get = core.get_h(), set = core.set_h(data))]
    H = 0b100,
    #[assoc(mneumonic = "l", get = core.get_l(), set = core.set_l(data))]
    L = 0b101,
    #[assoc(mneumonic = "[hl]", get = core.read(core.get_hl()), set = core.write(core.get_hl(), data))]
    HLmem = 0b110,
    #[assoc(mneumonic = "a", get = core.get_a(), set = core.set_a(data))]
    A = 0b111,
}

impl Display for R8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mneumonic())
    }
}

#[derive(Debug, Specifier, Assoc)]
#[func(pub fn mneumonic(&self) -> &str)]
#[func(pub fn get(&self, core: &Core) -> u16)]
#[func(pub fn set(&self, core: &mut Core, data: u16))]
pub enum R16 {
    #[assoc(mneumonic = "bc", get = core.get_bc(), set = core.set_bc(data))]
    BC = 0b00,
    #[assoc(mneumonic = "de", get = core.get_de(), set = core.set_de(data))]
    DE = 0b01,
    #[assoc(mneumonic = "hl", get = core.get_hl(), set = core.set_hl(data))]
    HL = 0b10,
    #[assoc(mneumonic = "sp", get = core.get_sp(), set = core.set_sp(data))]
    SP = 0b11,
}

impl Display for R16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mneumonic())
    }
}

#[derive(Debug, Specifier, Assoc)]
#[func(pub fn mneumonic(&self) -> &str)]
#[func(pub fn get(&self, core: &Core) -> u16)]
#[func(pub fn set(&self, core: &mut Core, data: u16))]
pub enum R16stk {
    #[assoc(mneumonic = "bc", get = core.get_bc(), set = core.set_bc(data))]
    BC = 0b00,
    #[assoc(mneumonic = "de", get = core.get_de(), set = core.set_de(data))]
    DE = 0b01,
    #[assoc(mneumonic = "hl", get = core.get_hl(), set = core.set_hl(data))]
    HL = 0b10,
    #[assoc(mneumonic = "af", get = core.get_af(), set = core.set_af(data))]
    AF = 0b11,
}
impl Display for R16stk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mneumonic())
    }
}
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Specifier, Assoc)]
#[func(pub fn mneumonic(&self) -> &str)]
#[func(pub fn get(&self, core: &mut Core) -> u16)]
pub enum R16mem {
    #[assoc(mneumonic = "bc", get = core.get_bc())]
    BC = 0b00,
    #[assoc(mneumonic = "de", get = core.get_de())]
    DE = 0b01,
    #[assoc(mneumonic = "hl+", get = {
        let hl = core.get_hl();
        core.set_hl(hl.wrapping_add(1));
        hl
    })]
    HLI = 0b10,
    #[assoc(mneumonic = "hl-", get = {
        let hl = core.get_hl();
        core.set_hl(hl.wrapping_sub(1));
        hl
    })]
    HLD = 0b11,
}

impl Display for R16mem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mneumonic())
    }
}

#[derive(Debug, Specifier, Assoc)]
#[func(pub fn mneumonic(&self) -> &str)]
#[func(pub fn check(&self, core: &Core) -> bool)]
pub enum Cond {
    #[assoc(mneumonic = "nz", check = !core.get_z_flag())]
    NZ = 0b00,
    #[assoc(mneumonic = "z", check = core.get_z_flag())]
    Z = 0b01,
    #[assoc(mneumonic = "nc", check = !core.get_c_flag())]
    NC = 0b10,
    #[assoc(mneumonic = "c", check = core.get_c_flag())]
    C = 0b11,
}

impl Display for Cond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.mneumonic())
    }
}

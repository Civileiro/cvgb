use std::rc::Rc;

use modular_bitfield::prelude::*;

use super::{bus::Bus, instruction::Instruction};

/// SM83 Core
/// Holds teh entire CPU state
pub struct Core {
    /// Current cycle count
    cycles: u64,
    /// Instruction cycle
    instruction_cycle: u8,
    /// Registers
    r: [u16; 6],
    /// Address Bus
    bus: Rc<Bus>,
    /// Interrupt master enable flag
    ime: bool,
    /// EI delay
    ei_delay: bool,
}

impl Core {
    pub fn new(bus: Rc<Bus>) -> Self {
        Self {
            cycles: 0,
            instruction_cycle: 0,
            r: [0; 6],
            bus,
            ime: false,
            ei_delay: false,
        }
    }
    pub fn get_af(&self) -> u16 {
        self.r[0]
    }
    pub fn set_af(&mut self, data: u16) {
        self.r[0] = data
    }
    pub fn get_bc(&self) -> u16 {
        self.r[1]
    }
    pub fn set_bc(&mut self, data: u16) {
        self.r[1] = data
    }
    pub fn get_de(&self) -> u16 {
        self.r[2]
    }
    pub fn set_de(&mut self, data: u16) {
        self.r[2] = data
    }
    pub fn get_hl(&self) -> u16 {
        self.r[3]
    }
    pub fn set_hl(&mut self, data: u16) {
        self.r[3] = data
    }
    pub fn get_a(&self) -> u8 {
        (self.get_af() >> 8) as u8
    }
    pub fn set_a(&mut self, data: u8) {
        self.set_af(((data as u16) << 8) | self.get_f() as u16)
    }
    pub fn get_f(&self) -> u8 {
        self.get_af() as u8
    }
    pub fn set_f(&mut self, data: u8) {
        self.set_af(((self.get_a() as u16) << 8) | (data as u16))
    }
    pub fn get_flags(&self) -> Flags {
        Flags::from_bytes([self.get_af() as u8])
    }
    pub fn set_flags(&mut self, data: Flags) {
        self.set_f(u8::from_bytes(data.into_bytes()[0]).unwrap())
    }
    pub fn get_b(&self) -> u8 {
        (self.get_bc() >> 8) as u8
    }
    pub fn set_b(&mut self, data: u8) {
        self.set_bc(((data as u16) << 8) | self.get_c() as u16);
    }
    pub fn get_c(&self) -> u8 {
        self.get_bc() as u8
    }
    pub fn set_c(&mut self, data: u8) {
        self.set_bc(((self.get_b() as u16) << 8) | (data as u16))
    }
    pub fn get_d(&self) -> u8 {
        (self.get_de() >> 8) as u8
    }
    pub fn set_d(&mut self, data: u8) {
        self.set_de(((data as u16) << 8) | self.get_e() as u16)
    }
    pub fn get_e(&self) -> u8 {
        self.get_de() as u8
    }
    pub fn set_e(&mut self, data: u8) {
        self.set_de(((self.get_d() as u16) << 8) | (data as u16))
    }
    pub fn get_h(&self) -> u8 {
        (self.get_hl() >> 8) as u8
    }
    pub fn set_h(&mut self, data: u8) {
        self.set_hl(((data as u16) << 8) | self.get_l() as u16)
    }
    pub fn get_l(&self) -> u8 {
        self.get_hl() as u8
    }
    pub fn set_l(&mut self, data: u8) {
        self.set_hl(((self.get_h() as u16) << 8) | (data as u16))
    }
    pub fn get_z_flag(&self) -> bool {
        self.get_flags().z()
    }
    pub fn set_z_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_z(data))
    }
    pub fn get_n_flag(&self) -> bool {
        self.get_flags().n()
    }
    pub fn set_n_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_n(data))
    }
    pub fn get_h_flag(&self) -> bool {
        self.get_flags().h()
    }
    pub fn set_h_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_h(data))
    }
    pub fn get_c_flag(&self) -> bool {
        self.get_flags().c()
    }
    pub fn set_c_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_c(data))
    }
    /// Read stack pointer (register 4)
    pub fn get_sp(&self) -> u16 {
        self.r[4]
    }
    pub fn set_sp(&mut self, data: u16) {
        self.r[4] = data
    }
    /// Read program counter (register 5)
    pub fn get_pc(&self) -> u16 {
        self.r[5]
    }
    pub fn set_pc(&mut self, data: u16) {
        self.r[5] = data
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }
    pub fn write(&self, addr: u16, data: u8) {
        self.bus.write(addr, data)
    }
    pub fn ei(&mut self) {
        self.ei_delay = true;
    }
    pub fn ei_instantly(&mut self) {
        self.ime = true;
    }
    pub fn di(&mut self) {
        self.ime = false;
    }
    pub fn irq(&mut self) {
        if !self.ime {
            return;
        }
        todo!()
    }
    pub fn get_ime(&self) -> bool {
        self.ime
    }

    pub fn clock(&mut self) {
        if self.instruction_cycle == 0 {
            if self.ei_delay {
                self.ime = true;
                self.ei_delay = false;
            }
            let Ok(instruction) = Instruction::decode(self) else {
                unimplemented!()
            };
            self.set_pc(self.get_pc().wrapping_add(instruction.size() as u16));
            self.instruction_cycle = instruction.execute(self);
            assert!(self.instruction_cycle != 0);
        }
        self.cycles += 1;
        self.instruction_cycle -= 1;
    }

    pub fn is_executing_instruction(&self) -> bool {
        self.instruction_cycle != 0
    }
}

#[bitfield(bits = 8)]
pub struct Flags {
    #[skip]
    __: B4,
    pub c: bool,
    pub h: bool,
    pub n: bool,
    pub z: bool,
}

use instructions::Execute;
use opcode::Opcode;
use registers::Registers;

use super::context::interrupts::{Interrupt, InterruptFlags};

mod decode;
mod instructions;
pub mod opcode;
pub mod registers;
#[cfg(test)]
mod tests;

/// SM83 Core
/// Holds teh entire CPU state
#[derive(Debug, Default)]
pub struct Cpu {
    /// Registers
    regs: Registers,
    /// Interrupt master enable flag
    ime: bool,
    /// Fetched opcode
    opcode: Opcode,
    /// Requested interrupts
    rqst_itrs: InterruptFlags,
}

pub trait CpuContext {
    fn cycle_read_itrs(&mut self, addr: u16) -> (u8, InterruptFlags);
    fn cycle_read(&mut self, addr: u16) -> u8 {
        self.cycle_read_itrs(addr).0
    }
    fn cycle_write(&mut self, addr: u16, data: u8);
    fn cycle(&mut self);
    fn ack_interrupt(&mut self, itr: Interrupt);
}

impl Cpu {
    pub fn step(&mut self, ctx: &mut impl CpuContext) {
        if let Some(interrupt) = self.rqst_itrs.highest_priority() {
            ctx.ack_interrupt(interrupt);
            todo!()
        } else {
            let opcode = self.opcode;
            self.execute(ctx, opcode);
        }
    }
    pub fn cycle(&self, ctx: &mut impl CpuContext) {
        ctx.cycle();
    }
    pub fn cycle_prefetch(&mut self, ctx: &mut impl CpuContext) {
        let (opcode, rqst_itrs) = ctx.cycle_read_itrs(self.regs.pc);
        self.opcode = Opcode::lookup(opcode);
        if self.ime {
            self.rqst_itrs = rqst_itrs;
        }
    }
    pub fn cycle_read(&self, ctx: &mut impl CpuContext, addr: u16) -> u8 {
        ctx.cycle_read(addr)
    }
    pub fn cycle_read_pc(&mut self, ctx: &mut impl CpuContext) -> u8 {
        let addr = self.regs.pc;
        self.regs.inc_pc();
        self.cycle_read(ctx, addr)
    }
    pub fn cycle_write(&self, ctx: &mut impl CpuContext, addr: u16, data: u8) {
        ctx.cycle_write(addr, data);
    }
    pub fn cycle_push16(&mut self, ctx: &mut impl CpuContext, data: u16) {
        let [hi, lo] = data.to_be_bytes();
        self.cycle(ctx);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.cycle_write(ctx, self.regs.sp, hi);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.cycle_write(ctx, self.regs.sp, lo);
    }
    pub fn cycle_pop16(&mut self, ctx: &mut impl CpuContext) -> u16 {
        let lo = self.cycle_read(ctx, self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        let hi = self.cycle_read(ctx, self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(1);
        u16::from_be_bytes([hi, lo])
    }

    pub fn stop(&mut self, ctx: &mut impl CpuContext) {
        todo!()
    }
    pub fn halt(&mut self, ctx: &mut impl CpuContext) {
        todo!()
    }
}

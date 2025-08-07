use instructions::{Execute, InputU8};
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
/// Holds the entire CPU state
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
    /// Current state
    state: CPUState,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CPUState {
    #[default]
    Normal,
    // In HALT mode the CPU does nothing while waiting for an interrupt
    // The CPU also enters HALT mode after a speed switch, in which case
    // it'll automatically exit after a certain amount of cycles
    Halt(u32),
    // In STOP mode the CPU does nothing while waiting for input
    Stop,
}

impl CPUState {
    pub fn reset(&mut self) {
        *self = Self::Normal
    }
    pub fn set_halt(&mut self) {
        *self = Self::Halt(0)
    }
    pub fn set_halt_timer(&mut self, timer: u32) {
        *self = Self::Halt(timer)
    }
    pub fn dec_halt_timer(&mut self) -> bool {
        if let Self::Halt(timer) = self {
            if *timer == 0 {
                return false;
            }
            *timer -= 1;
            if *timer == 0 {
                return true;
            }
        }
        false
    }
    pub fn set_stop(&mut self) {
        *self = Self::Stop
    }
    pub fn is_halt(&self) -> bool {
        matches!(self, Self::Halt(_))
    }
    pub fn is_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }
}

pub trait CpuContext {
    /// Cycle the context, reading from an address and returning interrupts to service
    fn cycle_read_itrs(&mut self, addr: u16) -> (u8, InterruptFlags);
    /// Cycle the context, reading from an address
    fn cycle_read(&mut self, addr: u16) -> u8 {
        self.cycle_read_itrs(addr).0
    }
    /// Cycle the context, writing into an address and returning interrupts to service
    fn cycle_write_itrs(&mut self, addr: u16, data: u8) -> InterruptFlags;
    /// Cycle the context, writing into an address
    fn cycle_write(&mut self, addr: u16, data: u8) {
        self.cycle_write_itrs(addr, data);
    }
    /// Cycle the context while in a special state
    fn cycle_state_itrs(&mut self, state: CPUState) -> InterruptFlags;
    /// Cycle the context
    fn cycle(&mut self) {
        self.cycle_state_itrs(CPUState::Normal);
    }
    /// Confirm that an interrupt was serviced
    fn ack_interrupt(&mut self, itr: Interrupt);
    /// Check if theres a pending interrupt
    fn has_interrupt(&mut self) -> bool;
    /// Switch context speed (CGB)
    fn speed_switch(&mut self);
    /// Check input line for a pressed button
    fn has_pressed_input(&self) -> bool;
}

impl Cpu {
    pub fn step(&mut self, ctx: &mut impl CpuContext) {
        if self.state.is_stop() {
            // In STOP mode the CPU does nothing while waiting for input
            ctx.cycle_state_itrs(self.state);
            if ctx.has_pressed_input() {
                self.state.reset();
            }
        } else if self.rqst_itrs.has_interrupt() {
            self.rqst_itrs.clear();
            // Servicing an interrupt disables HALT if it was active
            self.state.reset();
            // IME is disabled, its usually re-enabled when the handler calls RETI
            self.ime = false;

            // Interrupt servicing takes 5 cycles
            // 1: Decrement PC
            // Interrupt servicing happens after fetching the next opcode
            // As that wont be executed rn, we need to adjust the program counter
            self.regs.dec_pc();

            // 2: Decrement SP
            self.cycle(ctx);
            self.regs.sp = self.regs.sp.wrapping_sub(1);

            let [hi, lo] = self.regs.pc.to_be_bytes();
            // 3: Write PC.high to SP, decrement SP
            self.cycle_write(ctx, self.regs.sp, hi);
            self.regs.sp = self.regs.sp.wrapping_sub(1);

            // 4: Write PC.low to SP, set PC to the interrupt handler
            // The actual interrupt being serviced only matters in this cycle
            let interrupts = ctx.cycle_write_itrs(self.regs.sp, lo);
            if let Some(interrupt) = interrupts.highest_priority() {
                self.regs.pc = interrupt.handler_address();
                ctx.ack_interrupt(interrupt);
            } else {
                // Bugged interrupt?
                self.regs.pc = 0
            }

            // 5: Generic fetch
            self.opcode = Opcode::lookup(self.cycle_read_pc(ctx));
        } else if self.state.is_halt() {
            // In HALT mode the CPU does nothing while waiting for an interrupt
            self.rqst_itrs = ctx.cycle_state_itrs(self.state);
            // May also exit with a timer
            if self.state.dec_halt_timer() {
                self.state.reset();
            }
        } else {
            let opcode = self.opcode;
            self.execute(ctx, opcode);
        }
    }
    pub fn cycle_prefetch(&mut self, ctx: &mut impl CpuContext) {
        let pc = self.regs.pc;
        let (next_opcode, rqst_itrs) = ctx.cycle_read_itrs(pc);
        // If there are interrupts pending after executing HALT, the halt bug happens
        // making the program counter fail to increment
        let halt_bug = matches!(self.opcode, Opcode::HALT) && rqst_itrs.has_interrupt();
        if !halt_bug {
            self.regs.inc_pc();
        }
        self.opcode = Opcode::lookup(next_opcode);
        if self.ime {
            self.rqst_itrs = rqst_itrs;
        }
    }
    // The halt function is here and not in instructions.rs bc it's logic is
    // entangled with cycle_prefetch and the halt bug
    pub fn halt(&mut self, ctx: &mut impl CpuContext) {
        self.state.set_halt();
        self.cycle_prefetch(ctx);
    }
    fn cycle(&self, ctx: &mut impl CpuContext) {
        ctx.cycle();
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
    /// Push 16 bit value to the stack, takes 3 cycles
    pub fn cycle_push16(&mut self, ctx: &mut impl CpuContext, data: u16) {
        let [hi, lo] = data.to_be_bytes();
        self.cycle(ctx);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.cycle_write(ctx, self.regs.sp, hi);
        self.regs.sp = self.regs.sp.wrapping_sub(1);
        self.cycle_write(ctx, self.regs.sp, lo);
    }
    /// Pop 16 bit value from the stack, takes 2 cycles
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
}

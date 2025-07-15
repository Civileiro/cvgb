use std::fmt::Debug;

use super::{
    Cpu, CpuContext,
    opcode::{CBOpcode, Condition, Opcode},
    registers::{Reg8, Reg16},
};

pub trait Execute<T: Copy> {
    fn execute(&mut self, ctx: &mut impl CpuContext, opcode: T);
}

impl Execute<Opcode> for Cpu {
    fn execute(&mut self, ctx: &mut impl CpuContext, opcode: Opcode) {
        use Reg8::{A, C};
        match opcode {
            Opcode::NOP => self.cycle_prefetch(ctx),
            Opcode::LD_r16_imm16 { dest } => self.ld16(ctx, dest, Imm16),
            Opcode::LD_r16mem_a { r16mem } => self.ld8(ctx, r16mem, A),
            Opcode::LD_a_r16mem { r16mem } => self.ld8(ctx, A, r16mem),
            Opcode::LD_imm16_sp => self.ld_imm16_sp(ctx),
            Opcode::INC_r16 { r16 } => self.inc16(ctx, r16),
            Opcode::DEC_r16 { r16 } => self.dec16(ctx, r16),
            Opcode::ADD_hl_r16 { r16 } => self.add_hl(ctx, r16),
            Opcode::INC_r8 { r8 } => self.inc(ctx, r8),
            Opcode::DEC_r8 { r8 } => self.dec(ctx, r8),
            Opcode::LD_r8_imm8 { r8 } => self.ld8(ctx, r8, Imm8),
            Opcode::RLCA => self.rlca(ctx),
            Opcode::RRCA => self.rrca(ctx),
            Opcode::RLA => self.rla(ctx),
            Opcode::RRA => self.rra(ctx),
            Opcode::DAA => self.daa(ctx),
            Opcode::CPL => self.cpl(ctx),
            Opcode::SCF => self.scf(ctx),
            Opcode::CCF => self.ccf(ctx),
            Opcode::JR_imm8 => self.jr(ctx, None, Imm8),
            Opcode::JR_cond_imm8 { cond } => self.jr(ctx, Some(cond), Imm8),
            Opcode::STOP => self.stop(ctx),
            Opcode::LD_r8_r8 { dest, src } => {
                if dest != src {
                    self.ld8(ctx, dest, src)
                } else {
                    self.cycle_prefetch(ctx);
                }
            }
            Opcode::HALT => self.halt(ctx),
            Opcode::ADD_a_r8 { r8 } => self.add(ctx, r8),
            Opcode::ADC_a_r8 { r8 } => self.adc(ctx, r8),
            Opcode::SUB_a_r8 { r8 } => self.sub(ctx, r8),
            Opcode::SBC_a_r8 { r8 } => self.sbc(ctx, r8),
            Opcode::AND_a_r8 { r8 } => self.and(ctx, r8),
            Opcode::XOR_a_r8 { r8 } => self.xor(ctx, r8),
            Opcode::OR_a_r8 { r8 } => self.or(ctx, r8),
            Opcode::CP_a_r8 { r8 } => self.cp(ctx, r8),
            Opcode::ADD_a_imm8 => self.add(ctx, Imm8),
            Opcode::ADC_a_imm8 => self.adc(ctx, Imm8),
            Opcode::SUB_a_imm8 => self.sub(ctx, Imm8),
            Opcode::SBC_a_imm8 => self.sbc(ctx, Imm8),
            Opcode::AND_a_imm8 => self.and(ctx, Imm8),
            Opcode::XOR_a_imm8 => self.xor(ctx, Imm8),
            Opcode::OR_a_imm8 => self.or(ctx, Imm8),
            Opcode::CP_a_imm8 => self.cp(ctx, Imm8),
            Opcode::RET_cond { cond } => self.ret(ctx, Some(cond)),
            Opcode::RET => self.ret(ctx, None),
            Opcode::RETI => self.reti(ctx),
            Opcode::JP_cond_imm16 { cond } => self.jp(ctx, Some(cond), Imm16),
            Opcode::JP_imm16 => self.jp(ctx, None, Imm16),
            Opcode::JP_hl => self.jp_hl(ctx),
            Opcode::CALL_cond_imm16 { cond } => self.call(ctx, Some(cond), Imm16),
            Opcode::CALL_imm16 => self.call(ctx, None, Imm16),
            Opcode::RST { tgt3 } => self.call(ctx, None, tgt3 as u16 * 8),
            Opcode::POP { r16stk } => self.pop(ctx, r16stk),
            Opcode::PUSH { r16stk } => self.push(ctx, r16stk),
            Opcode::PREFIX => self.cb_prefix(ctx),
            Opcode::LDH_c_a => self.ld8(ctx, Addr(High(C)), A),
            Opcode::LDH_imm8_a => self.ld8(ctx, Addr(High(Imm8)), A),
            Opcode::LD_imm16_a => self.ld8(ctx, Addr(Imm16), A),
            Opcode::LDH_a_c => self.ld8(ctx, A, Addr(High(C))),
            Opcode::LDH_a_imm8 => self.ld8(ctx, A, Addr(High(Imm8))),
            Opcode::LD_a_imm16 => self.ld8(ctx, A, Addr(Imm16)),
            Opcode::ADD_sp_imm8 => self.add_sp_imm8(ctx),
            Opcode::LD_hl_spimm8 => self.ld_hl_spimm8(ctx),
            Opcode::LD_sp_hl => self.ld_sp_hl(ctx),
            Opcode::DI => self.di(ctx),
            Opcode::EI => self.ei(ctx),
            Opcode::INVALID => todo!(),
        }
    }
}

impl Execute<CBOpcode> for Cpu {
    fn execute(&mut self, ctx: &mut impl CpuContext, opcode: CBOpcode) {
        match opcode {
            CBOpcode::RLC { r8 } => self.rlc(ctx, r8),
            CBOpcode::RRC { r8 } => self.rrc(ctx, r8),
            CBOpcode::RL { r8 } => self.rl(ctx, r8),
            CBOpcode::RR { r8 } => self.rr(ctx, r8),
            CBOpcode::SLA { r8 } => self.sla(ctx, r8),
            CBOpcode::SRA { r8 } => self.sra(ctx, r8),
            CBOpcode::SWAP { r8 } => self.swap(ctx, r8),
            CBOpcode::SRL { r8 } => self.srl(ctx, r8),
            CBOpcode::BIT { b3, r8 } => self.bit(ctx, r8, b3),
            CBOpcode::RES { b3, r8 } => self.res(ctx, r8, b3),
            CBOpcode::SET { b3, r8 } => self.set(ctx, r8, b3),
        }
    }
}

pub trait InputU8<T: Copy> {
    fn read(&mut self, ctx: &mut impl CpuContext, input: T) -> u8;
}
pub trait OutputU8<T: Copy> {
    fn write(&mut self, ctx: &mut impl CpuContext, output: T, data: u8);
}
pub trait InputU16<T: Copy> {
    fn read16(&mut self, ctx: &mut impl CpuContext, input: T) -> u16;
}
pub trait OutputU16<T: Copy> {
    fn write16(&mut self, ctx: &mut impl CpuContext, output: T, data: u16);
}

#[derive(Debug, Clone, Copy)]
struct Imm8;

impl InputU8<Imm8> for Cpu {
    fn read(&mut self, ctx: &mut impl CpuContext, _: Imm8) -> u8 {
        self.cycle_read_pc(ctx)
    }
}

#[derive(Debug, Clone, Copy)]
struct Imm16;

impl InputU16<Imm16> for Cpu {
    fn read16(&mut self, ctx: &mut impl CpuContext, _: Imm16) -> u16 {
        let lo = self.cycle_read_pc(ctx);
        let hi = self.cycle_read_pc(ctx);
        u16::from_le_bytes([lo, hi])
    }
}

impl InputU16<u16> for Cpu {
    fn read16(&mut self, _: &mut impl CpuContext, input: u16) -> u16 {
        input
    }
}

#[derive(Clone, Copy)]
struct Addr<T: Copy>(T);

impl<T: Copy> InputU8<Addr<T>> for Cpu
where
    Self: InputU16<T>,
{
    fn read(&mut self, ctx: &mut impl CpuContext, input: Addr<T>) -> u8 {
        let addr = self.read16(ctx, input.0);
        self.cycle_read(ctx, addr)
    }
}
impl<T: Copy> OutputU8<Addr<T>> for Cpu
where
    Self: InputU16<T>,
{
    fn write(&mut self, ctx: &mut impl CpuContext, output: Addr<T>, data: u8) {
        let addr = self.read16(ctx, output.0);
        self.cycle_write(ctx, addr, data);
    }
}

#[derive(Clone, Copy)]
struct High<T: Copy>(T);

impl<T: Copy> InputU16<High<T>> for Cpu
where
    Self: InputU8<T>,
{
    fn read16(&mut self, ctx: &mut impl CpuContext, input: High<T>) -> u16 {
        0xFF00 | (self.read(ctx, input.0) as u16)
    }
}

impl Cpu {
    pub fn ld8<D: Copy, S: Copy>(&mut self, ctx: &mut impl CpuContext, dst: D, src: S)
    where
        Self: InputU8<S> + OutputU8<D>,
    {
        let data = self.read(ctx, src);
        self.write(ctx, dst, data);
        self.cycle_prefetch(ctx);
    }
    pub fn ld16<D: Copy, S: Copy>(&mut self, ctx: &mut impl CpuContext, dst: D, src: S)
    where
        Self: InputU16<S> + OutputU16<D>,
    {
        let data = self.read16(ctx, src);
        self.write16(ctx, dst, data);
        self.cycle_prefetch(ctx);
    }
    pub fn jr<T: Copy>(&mut self, ctx: &mut impl CpuContext, cond: Option<Condition>, input: T)
    where
        Self: InputU8<T>,
    {
        let adj = self.read(ctx, input);
        if let Some(cond) = cond
            && self.check_cond(cond)
        {
            self.regs.pc = self.regs.pc.wrapping_add(adj as u16);
            self.cycle(ctx);
        }
        self.cycle_prefetch(ctx);
    }
    pub fn jp<T: Copy>(&mut self, ctx: &mut impl CpuContext, cond: Option<Condition>, input: T)
    where
        Self: InputU16<T>,
    {
        let addr = self.read16(ctx, input);
        if cond.is_none_or(|cond| self.check_cond(cond)) {
            self.regs.pc = addr;
            self.cycle(ctx);
        }
        self.cycle_prefetch(ctx);
    }
    pub fn jp_hl(&mut self, ctx: &mut impl CpuContext) {
        self.regs.pc = self.regs.get16(Reg16::HL);
        self.cycle_prefetch(ctx);
    }
    pub fn ret(&mut self, ctx: &mut impl CpuContext, cond: Option<Condition>) {
        if cond.is_some() {
            // The conditional version takes 1 cycle to check it
            self.cycle(ctx);
        }
        if cond.is_none_or(|cond| self.check_cond(cond)) {
            // 1 + 2
            let addr = self.cycle_pop16(ctx);
            // 3
            self.regs.pc = addr;
            self.cycle(ctx);
        }
        self.cycle_prefetch(ctx);
    }
    pub fn reti(&mut self, ctx: &mut impl CpuContext) {
        self.ime = true;
        self.ret(ctx, None);
    }
    pub fn call<T: Copy>(&mut self, ctx: &mut impl CpuContext, cond: Option<Condition>, input: T)
    where
        Self: InputU16<T>,
    {
        let addr = self.read16(ctx, input);
        if cond.is_none_or(|cond| self.check_cond(cond)) {
            self.cycle_push16(ctx, self.regs.pc);
            self.regs.pc = addr;
        }
        self.cycle_prefetch(ctx);
    }
    pub fn pop<T: Copy>(&mut self, ctx: &mut impl CpuContext, output: T)
    where
        Self: OutputU16<T>,
    {
        let data = self.cycle_pop16(ctx);
        self.write16(ctx, output, data);
        self.cycle_prefetch(ctx);
    }
    pub fn push<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU16<T>,
    {
        let data = self.read16(ctx, input);
        self.cycle_push16(ctx, data);
        self.cycle_prefetch(ctx);
    }
    pub fn di(&mut self, ctx: &mut impl CpuContext) {
        self.ime = false;
        self.cycle_prefetch(ctx);
    }
    pub fn ei(&mut self, ctx: &mut impl CpuContext) {
        self.cycle_prefetch(ctx);
        self.ime = true;
    }
    pub fn add<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let a = self.regs.a;
        let (res, c) = a.overflowing_add(val);
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag((a & 0x0F) + (val & 0x0F) > 0x0F);
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }
    pub fn adc<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let old = self.regs.a;
        let (new, c_val) = old.overflowing_add(val);
        let c = self.regs.get_c_flag() as u8;
        let (res, c_c) = new.overflowing_add(c);
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag((old & 0x0F) + (val & 0x0F) + c > 0x0F);
        self.regs.set_c_flag(c_val || c_c);
        self.cycle_prefetch(ctx);
    }

    pub fn sub<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let old = self.regs.a;
        let (res, c) = old.overflowing_sub(val);
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(true);
        self.regs.set_h_flag((val & 0x0F) > (old & 0x0F));
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }

    pub fn sbc<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let old = self.regs.a;
        let (new, c_val) = old.overflowing_sub(val);
        let c = self.regs.get_c_flag() as u8;
        let (res, c_c) = new.overflowing_sub(c);
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(true);
        self.regs.set_h_flag(((val & 0x0F) + c) > (old & 0x0F));
        self.regs.set_c_flag(c_val || c_c);
        self.cycle_prefetch(ctx);
    }
    pub fn and<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let res = self.regs.a & val;
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(true);
        self.regs.set_c_flag(false);
        self.cycle_prefetch(ctx);
    }
    pub fn xor<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let res = self.regs.a ^ val;
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(false);
        self.cycle_prefetch(ctx);
    }
    pub fn or<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let res = self.regs.a | val;
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(false);
        self.cycle_prefetch(ctx);
    }
    pub fn cp<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let a = self.regs.a;
        self.regs.set_z_flag(a == val);
        self.regs.set_n_flag(true);
        self.regs.set_h_flag((val & 0x0F) > (a & 0x0F));
        self.regs.set_c_flag(val > a);
        self.cycle_prefetch(ctx);
    }
    pub fn inc<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let res = val.wrapping_add(1);
        self.write(ctx, inoutput, res);
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag((val & 0x0F) + (res & 0x0F) > 0x0F);
        self.cycle_prefetch(ctx);
    }
    pub fn inc16<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU16<T> + OutputU16<T>,
    {
        let res = self.read16(ctx, inoutput).wrapping_add(1);
        self.write16(ctx, inoutput, res);
        self.cycle(ctx);
        self.cycle_prefetch(ctx);
    }
    pub fn dec<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let res = val.wrapping_sub(1);
        self.write(ctx, inoutput, res);
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(true);
        self.regs.set_h_flag((val & 0x0F) > (res & 0x0F));
        self.cycle_prefetch(ctx);
    }
    pub fn dec16<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU16<T> + OutputU16<T>,
    {
        let res = self.read16(ctx, inoutput).wrapping_sub(1);
        self.write16(ctx, inoutput, res);
        self.cycle(ctx);
        self.cycle_prefetch(ctx);
    }
    pub fn cpl(&mut self, ctx: &mut impl CpuContext) {
        self.regs.a = !self.regs.a;
        self.regs.set_n_flag(true);
        self.regs.set_h_flag(true);
        self.cycle_prefetch(ctx);
    }
    pub fn scf(&mut self, ctx: &mut impl CpuContext) {
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(true);
        self.cycle_prefetch(ctx);
    }
    pub fn ccf(&mut self, ctx: &mut impl CpuContext) {
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(!self.regs.get_c_flag());
        self.cycle_prefetch(ctx);
    }
    pub fn rlc<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let res = val.rotate_left(1);
        self.write(ctx, inoutput, res);
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(res & 1 == 1);
        self.cycle_prefetch(ctx);
    }
    pub fn rlca(&mut self, ctx: &mut impl CpuContext) {
        self.rlc(ctx, Reg8::A);
        self.regs.set_z_flag(false);
    }
    pub fn rrc<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let res = val.rotate_right(1);
        self.write(ctx, inoutput, res);
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(res & 1 == 1);
        self.cycle_prefetch(ctx);
    }
    pub fn rrca(&mut self, ctx: &mut impl CpuContext) {
        self.rrc(ctx, Reg8::A);
        self.regs.set_z_flag(false);
    }
    pub fn rl<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let rot = val.rotate_left(1);
        let res = rot & 0xFE | self.regs.get_c_flag() as u8;
        let c = rot & 1 == 1;
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }
    pub fn rla(&mut self, ctx: &mut impl CpuContext) {
        self.rl(ctx, Reg8::A);
        self.regs.set_z_flag(false);
    }
    pub fn rr<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let c = val & 1 == 1;
        let flp = val & 0xFE | self.regs.get_c_flag() as u8;
        let res = flp.rotate_left(1);
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }
    pub fn rra(&mut self, ctx: &mut impl CpuContext) {
        self.rr(ctx, Reg8::A);
        self.regs.set_z_flag(false);
    }
    pub fn daa(&mut self, ctx: &mut impl CpuContext) {
        if self.regs.get_n_flag() {
            let mut adj = 0;
            if self.regs.get_h_flag() {
                adj += 0x6
            }
            if self.regs.get_c_flag() {
                adj += 0x60
            }
            self.regs.a = self.regs.a.wrapping_sub(adj);
        } else {
            let mut adj = 0;
            let a = self.regs.a;
            if self.regs.get_h_flag() || a & 0xF > 0x9 {
                adj += 0x6
            }
            if self.regs.get_c_flag() || a > 0x99 {
                adj += 0x60;
                self.regs.set_c_flag(true);
            }
            self.regs.a = self.regs.a.wrapping_add(adj);
        }
        self.regs.set_z_flag(self.regs.a == 0);
        self.regs.set_h_flag(false);
        self.cycle_prefetch(ctx);
    }
    pub fn sla<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let c = val & 0x80 != 0;
        let res = val << 1;
        self.write(ctx, inoutput, res);
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }
    pub fn sra<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let c = val & 1 != 0;
        let b8 = val & 0x80;
        let res = (val >> 1) | b8;
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }
    pub fn swap<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let low = val & 0x0F;
        let high = val & 0xF0;
        let res = (low << 4) | (high >> 4);
        self.regs.a = res;
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(false);
        self.cycle_prefetch(ctx);
    }
    pub fn srl<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let c = val & 1 != 0;
        let res = val >> 1;
        self.write(ctx, inoutput, res);
        self.regs.set_z_flag(res == 0);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(false);
        self.regs.set_c_flag(c);
        self.cycle_prefetch(ctx);
    }
    pub fn bit<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T, b3: u8)
    where
        Self: InputU8<T>,
    {
        let val = self.read(ctx, input);
        let z = (val & (1 << b3)) == 0;
        self.regs.set_z_flag(z);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag(true);
        self.cycle_prefetch(ctx);
    }
    pub fn res<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T, b3: u8)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let mask = !(1 << b3);
        let res = val & mask;
        self.write(ctx, inoutput, res);
        self.cycle_prefetch(ctx);
    }
    pub fn set<T: Copy>(&mut self, ctx: &mut impl CpuContext, inoutput: T, b3: u8)
    where
        Self: InputU8<T> + OutputU8<T>,
    {
        let val = self.read(ctx, inoutput);
        let mask = 1 << b3;
        let res = val | mask;
        self.write(ctx, inoutput, res);
        self.cycle_prefetch(ctx);
    }
    pub fn ld_imm16_sp(&mut self, ctx: &mut impl CpuContext) {
        let addr = self.read16(ctx, Imm16);
        let sp = self.regs.sp;
        self.cycle_write(ctx, addr, sp as u8);
        self.cycle_write(ctx, addr.wrapping_add(1), (sp >> 8) as u8);
        self.cycle_prefetch(ctx);
    }
    pub fn add_hl<T: Copy>(&mut self, ctx: &mut impl CpuContext, input: T)
    where
        Self: InputU16<T>,
    {
        let val = self.read16(ctx, input);
        let hl = self.regs.get16(Reg16::HL);
        let (res, c) = hl.overflowing_add(val);
        self.regs.set16(Reg16::HL, res);
        self.regs.set_n_flag(false);
        self.regs
            .set_h_flag((hl & 0x0FFF) + (res & 0x0FFF) > 0x0FFF);
        self.regs.set_c_flag(c);
        self.cycle(ctx);
        self.cycle_prefetch(ctx);
    }

    pub fn add_sp_imm8(&mut self, ctx: &mut impl CpuContext) {
        let offset = self.read(ctx, Imm8) as i8 as u16;
        let sp = self.regs.sp;
        self.regs.sp = sp.wrapping_add(offset);
        self.regs.set_z_flag(false);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag((sp & 0x0F) + (offset & 0x0F) > 0x0F);
        self.regs.set_h_flag((sp & 0xFF) + (offset & 0xFF) > 0xFF);
        self.cycle(ctx);
        self.cycle(ctx);
        self.cycle_prefetch(ctx);
    }
    pub fn ld_hl_spimm8(&mut self, ctx: &mut impl CpuContext) {
        let offset = self.read(ctx, Imm8) as i8 as u16;
        let sp = self.regs.sp;
        self.regs.set16(Reg16::HL, sp.wrapping_add(offset));
        self.regs.set_z_flag(false);
        self.regs.set_n_flag(false);
        self.regs.set_h_flag((sp & 0x0F) + (offset & 0x0F) > 0x0F);
        self.regs.set_h_flag((sp & 0xFF) + (offset & 0xFF) > 0xFF);
        self.cycle(ctx);
        self.cycle_prefetch(ctx);
    }
    pub fn ld_sp_hl(&mut self, ctx: &mut impl CpuContext) {
        self.cycle(ctx);
        self.regs.set16(Reg16::HL, self.regs.sp);
        self.cycle_prefetch(ctx);
    }
    pub fn cb_prefix(&mut self, ctx: &mut impl CpuContext) {
        let cb_opcode = CBOpcode::lookup(self.read(ctx, Imm8));
        self.execute(ctx, cb_opcode);
    }
}

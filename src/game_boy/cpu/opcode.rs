use std::{
    fmt::{Display, Write},
    ops::Not,
};

use compact_str::{CompactString, format_compact};
use enum_assoc::Assoc;

use super::{
    Cpu, CpuContext,
    instructions::{InputU8, OutputU8},
    registers::{Reg8, Reg16},
};

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Debug, Assoc, Clone, Copy, Default)]
#[func(pub const fn instruction_size(&self) -> usize)]
#[func(pub fn mneumonic(&self) -> CompactString)]
pub enum Opcode {
    // Block 0
    #[default]
    #[assoc(instruction_size = 1, mneumonic = "nop".into())]
    NOP,
    #[assoc(instruction_size = 3, mneumonic = format_compact!("ld {_dest}, imm16"))]
    LD_r16_imm16 { dest: Reg16 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ld [{_r16mem}], a"))]
    LD_r16mem_a { r16mem: R16mem },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ld a, [{_r16mem}]"))]
    LD_a_r16mem { r16mem: R16mem },
    #[assoc(instruction_size = 3, mneumonic = format_compact!("ld [imm16], sp"))]
    LD_imm16_sp,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("inc {_r16}"))]
    INC_r16 { r16: Reg16 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("dec {_r16}"))]
    DEC_r16 { r16: Reg16 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("add hl, {_r16}"))]
    ADD_hl_r16 { r16: Reg16 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("inc {_r8}"))]
    INC_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("dec {_r8}"))]
    DEC_r8 { r8: R8 },
    #[assoc(instruction_size = 2, mneumonic = format_compact!("ld {_r8}, imm8"))]
    LD_r8_imm8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("rlca"))]
    RLCA,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("rrca"))]
    RRCA,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("rla"))]
    RLA,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("rra"))]
    RRA,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("daa"))]
    DAA,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("cpl"))]
    CPL,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("scf"))]
    SCF,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ccf"))]
    CCF,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("jr imm8"))]
    JR_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("jr {_cond}, imm8"))]
    JR_cond_imm8 { cond: Condition },
    #[assoc(instruction_size = 2, mneumonic = format_compact!("stop"))]
    STOP,
    // Block 1
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ld {_dest}, {_src}"))]
    LD_r8_r8 { dest: R8, src: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("halt"))]
    HALT,
    // Block 2
    #[assoc(instruction_size = 1, mneumonic = format_compact!("add a, {_r8}"))]
    ADD_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("adc a, {_r8}"))]
    ADC_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("sub a, {_r8}"))]
    SUB_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("sbc a, {_r8}"))]
    SBC_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("and a, {_r8}"))]
    AND_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("xor a, {_r8}"))]
    XOR_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("or a, {_r8}"))]
    OR_a_r8 { r8: R8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("cp a, {_r8}"))]
    CP_a_r8 { r8: R8 },
    // Block 3
    #[assoc(instruction_size = 2, mneumonic = format_compact!("add a, imm8"))]
    ADD_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("adc a, imm8"))]
    ADC_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("sub a, imm8"))]
    SUB_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("sbc a, imm8"))]
    SBC_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("and a, imm8"))]
    AND_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("xor a, imm8"))]
    XOR_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("or a, imm8"))]
    OR_a_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("cp a, imm8"))]
    CP_a_imm8,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ret {_cond}"))]
    RET_cond { cond: Condition },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ret"))]
    RET,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("reti"))]
    RETI,
    #[assoc(instruction_size = 3, mneumonic = format_compact!("jp {_cond}, imm16"))]
    JP_cond_imm16 { cond: Condition },
    #[assoc(instruction_size = 3, mneumonic = format_compact!("jp imm16"))]
    JP_imm16,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("jp hl"))]
    JP_hl,
    #[assoc(instruction_size = 3, mneumonic = format_compact!("call {_cond}, imm16"))]
    CALL_cond_imm16 { cond: Condition },
    #[assoc(instruction_size = 3, mneumonic = format_compact!("call imm16"))]
    CALL_imm16,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("rst"))]
    RST { tgt3: u8 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("pop"))]
    POP { r16stk: Reg16 },
    #[assoc(instruction_size = 1, mneumonic = format_compact!("push"))]
    PUSH { r16stk: Reg16 },
    #[assoc(instruction_size = 2, mneumonic = format_compact!("prefix"))]
    PREFIX,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ldh [c], a"))]
    LDH_c_a,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("ldh [imm8], a"))]
    LDH_imm8_a,
    #[assoc(instruction_size = 3, mneumonic = format_compact!("ld [imm16], a"))]
    LD_imm16_a,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ldh a, [c]"))]
    LDH_a_c,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("ldh a, [imm8]"))]
    LDH_a_imm8,
    #[assoc(instruction_size = 3, mneumonic = format_compact!("ld a, [imm16]"))]
    LD_a_imm16,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("add sp, imm8"))]
    ADD_sp_imm8,
    #[assoc(instruction_size = 2, mneumonic = format_compact!("ld hl, sp + imm8"))]
    LD_hl_spimm8,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ld sp, hl"))]
    LD_sp_hl,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("di"))]
    DI,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("ei"))]
    EI,
    #[assoc(instruction_size = 1, mneumonic = format_compact!("???"))]
    INVALID,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.mneumonic())
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Assoc, Clone, Copy)]
#[func(pub fn mneumonic(&self) -> CompactString)]
pub enum CBOpcode {
    #[assoc(mneumonic = format_compact!("rlc {_r8}"))]
    RLC { r8: R8 },
    #[assoc(mneumonic = format_compact!("rrc {_r8}"))]
    RRC { r8: R8 },
    #[assoc(mneumonic = format_compact!("rl {_r8}"))]
    RL { r8: R8 },
    #[assoc(mneumonic = format_compact!("rr {_r8}"))]
    RR { r8: R8 },
    #[assoc(mneumonic = format_compact!("sla {_r8}"))]
    SLA { r8: R8 },
    #[assoc(mneumonic = format_compact!("sra {_r8}"))]
    SRA { r8: R8 },
    #[assoc(mneumonic = format_compact!("swap {_r8}"))]
    SWAP { r8: R8 },
    #[assoc(mneumonic = format_compact!("srl {_r8}"))]
    SRL { r8: R8 },
    #[assoc(mneumonic = format_compact!("bit {_b3}, {_r8}"))]
    BIT { b3: u8, r8: R8 },
    #[assoc(mneumonic = format_compact!("res {_b3}, {_r8}"))]
    RES { b3: u8, r8: R8 },
    #[assoc(mneumonic = format_compact!("set {_b3}, {_r8}"))]
    SET { b3: u8, r8: R8 },
}

/// Reference to specific registers for use in Opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16mem {
    BC,
    DE,
    HLi,
    HLd,
}

impl Display for R16mem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use R16mem::*;
        match self {
            BC => f.write_str("bc"),
            DE => f.write_str("de"),
            HLi => f.write_str("hl+"),
            HLd => f.write_str("hl-"),
        }
    }
}

impl InputU8<R16mem> for Cpu {
    fn read(&mut self, ctx: &mut impl CpuContext, input: R16mem) -> u8 {
        use R16mem::*;
        match input {
            BC => {
                let addr = self.regs.get16(Reg16::BC);
                self.cycle_read(ctx, addr)
            }
            DE => {
                let addr = self.regs.get16(Reg16::DE);
                self.cycle_read(ctx, addr)
            }
            HLi => {
                let addr = self.regs.get16(Reg16::HL);
                let res = self.cycle_read(ctx, addr);
                self.regs.set16(Reg16::HL, addr.wrapping_add(1));
                res
            }
            HLd => {
                let addr = self.regs.get16(Reg16::HL);
                let res = self.cycle_read(ctx, addr);
                self.regs.set16(Reg16::HL, addr.wrapping_sub(1));
                res
            }
        }
    }
}
impl OutputU8<R16mem> for Cpu {
    fn write(&mut self, ctx: &mut impl CpuContext, output: R16mem, data: u8) {
        use R16mem::*;
        match output {
            BC => {
                let addr = self.regs.get16(Reg16::BC);
                self.cycle_write(ctx, addr, data);
            }
            DE => {
                let addr = self.regs.get16(Reg16::DE);
                self.cycle_write(ctx, addr, data);
            }
            HLi => {
                let addr = self.regs.get16(Reg16::HL);
                self.cycle_write(ctx, addr, data);
                self.regs.set16(Reg16::HL, addr.wrapping_add(1));
            }
            HLd => {
                let addr = self.regs.get16(Reg16::HL);
                self.cycle_write(ctx, addr, data);
                self.regs.set16(Reg16::HL, addr.wrapping_sub(1));
            }
        }
    }
}

/// Reference to 8-bit registers and [hl] for use in Opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R8 {
    Reg(Reg8),
    HLaddr,
}

impl Display for R8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use R8::*;
        match self {
            Reg(reg8) => reg8.fmt(f),
            HLaddr => f.write_str("[hl]"),
        }
    }
}

impl InputU8<R8> for Cpu {
    fn read(&mut self, ctx: &mut impl CpuContext, input: R8) -> u8 {
        use R8::*;
        match input {
            Reg(reg8) => self.regs.get8(reg8),
            HLaddr => {
                let hl = self.regs.get16(Reg16::HL);
                self.cycle_read(ctx, hl)
            }
        }
    }
}
impl OutputU8<R8> for Cpu {
    fn write(&mut self, ctx: &mut impl CpuContext, output: R8, data: u8) {
        use R8::*;
        match output {
            Reg(reg8) => self.regs.set8(reg8, data),
            HLaddr => {
                let hl = self.regs.get16(Reg16::HL);
                self.cycle_write(ctx, hl, data);
            }
        }
    }
}

/// impl CpuCondition for use in Opcodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::NZ => f.write_str("nz"),
            Condition::Z => f.write_char('z'),
            Condition::NC => f.write_str("nc"),
            Condition::C => f.write_char('c'),
        }
    }
}

impl Cpu {
    pub fn check_cond(&mut self, cond: Condition) -> bool {
        match cond {
            Condition::NZ => self.regs.get_z_flag().not(),
            Condition::Z => self.regs.get_z_flag(),
            Condition::NC => self.regs.get_c_flag().not(),
            Condition::C => self.regs.get_c_flag(),
        }
    }
}

mod decode;
mod disassemble;
mod execute;
mod groupings;

use compact_str::{CompactString, format_compact};
use enum_assoc::Assoc;
pub use groupings::{Cond, R8, R16, R16mem, R16stk};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Assoc)]
#[func(pub fn size(&self) -> u8 { 1 })]
#[func(pub fn mneumonic(&self) -> CompactString)]
pub enum Instruction {
    // Block 0
    #[assoc(mneumonic = "nop".into())]
    NOP,
    #[assoc( mneumonic = format_compact!("ld {}, {}", _dest, _imm16), size = 3,)]
    LDRimm { dest: R16, imm16: u16 },
    #[assoc( mneumonic = format_compact!("ld a, [{}]", _r16_mem), size = 3,)]
    LDRmem { r16_mem: R16mem },
    #[assoc( mneumonic = format_compact!("ld [{}], a", _r16_mem), size = 3,)]
    STRmem { r16_mem: R16mem },
    #[assoc( mneumonic = format_compact!("ld [{}], sp", _imm16_mem), size = 3,)]
    STRsp { imm16_mem: u16 },
    #[assoc(mneumonic = format_compact!("inc {}", _r16))]
    INCr16 { r16: R16 },
    #[assoc(mneumonic = format_compact!("dec {}", _r16))]
    DECr16 { r16: R16 },
    #[assoc(mneumonic = format_compact!("add hl, {}", _r16))]
    ADDhl { r16: R16 },
    #[assoc(mneumonic = format_compact!("inc {}", _r8))]
    INCr8 { r8: R8 },
    #[assoc(mneumonic = format_compact!("dec {}", _r8))]
    DECr8 { r8: R8 },
    #[assoc(mneumonic = format_compact!("ld {}, {}", _r8, _imm8), size = 2)]
    LDRr8 { r8: R8, imm8: u8 },
    #[assoc(mneumonic = "rlca".into())]
    RLCA,
    #[assoc(mneumonic = "rrca".into())]
    RRCA,
    #[assoc(mneumonic = "rla".into())]
    RLA,
    #[assoc(mneumonic = "rra".into())]
    RRA,
    #[assoc(mneumonic = "daa".into())]
    DAA,
    #[assoc(mneumonic = "cpl".into())]
    CPL,
    #[assoc(mneumonic = "scf".into())]
    SCF,
    #[assoc(mneumonic = "ccf".into())]
    CCF,
    #[assoc(mneumonic = format_compact!("jr {}", _imm8), size = 2)]
    JR { imm8: u8 },
    #[assoc(mneumonic = format_compact!("jr {}, {}", _cond, _imm8), size = 2)]
    JRcond { imm8: u8, cond: Cond },
    #[assoc(mneumonic = "stop".into(), size = 2)]
    STOP,
    // Block 1
    #[assoc(mneumonic = format_compact!("ld {}, {}", _dest, _src))]
    LDR { dest: R8, src: R8 },
    #[assoc(mneumonic = "halt".into())]
    HALT,
    // Block 2
    #[assoc(mneumonic = format_compact!("add a, {}", _r8))]
    ADD { r8: R8 },
    #[assoc(mneumonic = format_compact!("adc a, {}", _r8))]
    ADC { r8: R8 },
    #[assoc(mneumonic = format_compact!("sub a, {}", _r8))]
    SUB { r8: R8 },
    #[assoc(mneumonic = format_compact!("sbc a, {}", _r8))]
    SBC { r8: R8 },
    #[assoc(mneumonic = format_compact!("and a, {}", _r8))]
    AND { r8: R8 },
    #[assoc(mneumonic = format_compact!("xor a, {}", _r8))]
    XOR { r8: R8 },
    #[assoc(mneumonic = format_compact!("or a, {}", _r8))]
    OR { r8: R8 },
    #[assoc(mneumonic = format_compact!("cp a, {}", _r8))]
    CP { r8: R8 },
    // Block 3
    #[assoc(mneumonic = format_compact!("add a, {}", _imm8), size = 2)]
    ADDimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("adc a, {}", _imm8), size = 2)]
    ADCimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("sub a, {}", _imm8), size = 2)]
    SUBimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("sbc a, {}", _imm8), size = 2)]
    SBCimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("and a, {}", _imm8), size = 2)]
    ANDimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("xor a, {}", _imm8), size = 2)]
    XORimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("or a, {}", _imm8), size = 2)]
    ORimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("cp a, {}", _imm8), size = 2)]
    CPimm { imm8: u8 },
    #[assoc(mneumonic = format_compact!("ret {}", _cond))]
    RETcond { cond: Cond },
    #[assoc(mneumonic = "ret".into())]
    RET,
    #[assoc(mneumonic = "reti".into())]
    RETI,
    #[assoc(mneumonic = format_compact!("jp {}, {}", _cond, _imm16), size = 3)]
    JPcond { cond: Cond, imm16: u16 },
    #[assoc(mneumonic = format_compact!("jp {}", _imm16), size = 3)]
    JPimm { imm16: u16 },
    #[assoc(mneumonic = "jp hl".into())]
    JPhl,
    #[assoc(mneumonic = format_compact!("call {}, {}", _cond, _imm16), size = 3)]
    CALLcond { cond: Cond, imm16: u16 },
    #[assoc(mneumonic = format_compact!("call {}", _imm16), size = 3)]
    CALLimm { imm16: u16 },
    #[assoc(mneumonic = format_compact!("rst {}", _tgt3 * 8))]
    RST { tgt3: u8 },
    #[assoc(mneumonic = format_compact!("pop {}", _r16stk))]
    POP { r16stk: R16stk },
    #[assoc(mneumonic = format_compact!("push {}", _r16stk))]
    PUSH { r16stk: R16stk },
    #[assoc(mneumonic = format_compact!("rlc {}", _r8), size = 2)]
    RLC { r8: R8 },
    #[assoc(mneumonic = format_compact!("rrc {}", _r8), size = 2)]
    RRC { r8: R8 },
    #[assoc(mneumonic = format_compact!("rl {}", _r8), size = 2)]
    RL { r8: R8 },
    #[assoc(mneumonic = format_compact!("rr {}", _r8), size = 2)]
    RR { r8: R8 },
    #[assoc(mneumonic = format_compact!("sla {}", _r8), size = 2)]
    SLA { r8: R8 },
    #[assoc(mneumonic = format_compact!("sra {}", _r8), size = 2)]
    SRA { r8: R8 },
    #[assoc(mneumonic = format_compact!("swap {}", _r8), size = 2)]
    SWAP { r8: R8 },
    #[assoc(mneumonic = format_compact!("srl {}", _r8), size = 2)]
    SRL { r8: R8 },
    #[assoc(mneumonic = format_compact!("bit {}, {}", _b3, _r8), size = 2)]
    BIT { b3: u8, r8: R8 },
    #[assoc(mneumonic = format_compact!("res {}, {}", _b3, _r8), size = 2)]
    RES { b3: u8, r8: R8 },
    #[assoc(mneumonic = format_compact!("set {}, {}", _b3, _r8), size = 2)]
    SET { b3: u8, r8: R8 },
    #[assoc(mneumonic = "ldh [c], a".into())]
    STH,
    #[assoc(mneumonic = format_compact!("ldh [{}], a", _imm8), size = 2)]
    STHaddr { imm8: u8 },
    #[assoc(mneumonic = format_compact!("ld [{}], a", _imm16), size = 3)]
    STRaddr { imm16: u16 },
    #[assoc(mneumonic = "ldh a, [c]".into())]
    LDH,
    #[assoc(mneumonic = format_compact!("ldh a, [{}]", _imm8), size = 2)]
    LDHaddr { imm8: u8 },
    #[assoc(mneumonic = format_compact!("ld a, [{}]", _imm16), size = 3)]
    LDaddr { imm16: u16 },
    #[assoc(mneumonic = format_compact!("add sp, {}", _imm8), size = 2)]
    ADDsp { imm8: u8 },
    #[assoc(mneumonic = format_compact!("ld hl, sp + {}", _imm8), size = 2)]
    LDhl { imm8: u8 },
    #[assoc(mneumonic = "ld sp, hl".into())]
    LDsp,
    #[assoc(mneumonic = "di".into())]
    DI,
    #[assoc(mneumonic = "ei".into())]
    EI,
}

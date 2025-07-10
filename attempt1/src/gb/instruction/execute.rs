use crate::gb::core::Core;

use super::{Instruction, groupings::R8};

impl Instruction {
    /// Execute the instruction on the CPU, returning the number of cyles spent
    pub fn execute(&self, core: &mut Core) -> u8 {
        // println!(
        //     "Executing: ${:04x} {} ({:?})",
        //     core.get_pc(),
        //     self.mneumonic(),
        //     self
        // );
        match self {
            Instruction::NOP => 1,
            Instruction::LDRimm { dest, imm16 } => {
                dest.set(core, *imm16);
                3
            }
            Instruction::LDRmem { r16_mem } => {
                let addr = r16_mem.get(core);
                core.set_a(core.read(addr));
                2
            }
            Instruction::STRmem { r16_mem } => {
                let addr = r16_mem.get(core);
                core.write(addr, core.get_a());
                4
            }
            Instruction::STRsp { imm16_mem } => {
                core.write(*imm16_mem, core.get_sp() as u8);
                core.write(*imm16_mem + 1, (core.get_sp() >> 8) as u8);
                5
            }
            Instruction::INCr16 { r16 } => {
                r16.set(core, r16.get(core).wrapping_add(1));
                2
            }
            Instruction::DECr16 { r16 } => {
                r16.set(core, r16.get(core).wrapping_sub(1));
                2
            }
            Instruction::ADDhl { r16 } => {
                let hl = core.get_hl();
                let (new, c) = hl.overflowing_add(r16.get(core));
                core.set_hl(new);
                core.set_n_flag(false);
                core.set_h_flag((hl & 0x0FFF) + (new & 0x0FFF) > 0x0FFF);
                core.set_c_flag(c);
                2
            }
            Instruction::INCr8 { r8 } => {
                let val = r8.get(core);
                let res = val.wrapping_add(1);
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag((val & 0x0F) + (res & 0x0F) > 0x0F);
                1
            }
            Instruction::DECr8 { r8 } => {
                let val = r8.get(core);
                let res = val.wrapping_sub(1);
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(true);
                core.set_h_flag((val & 0x0F) > (res & 0x0F));
                1
            }
            Instruction::LDRr8 { r8, imm8 } => {
                r8.set(core, *imm8);
                2
            }
            Instruction::RLCA => {
                Instruction::RLC { r8: R8::A }.execute(core);
                core.set_z_flag(false);
                1
            }
            Instruction::RRCA => {
                Instruction::RRC { r8: R8::A }.execute(core);
                core.set_z_flag(false);
                1
            }
            Instruction::RLA => {
                Instruction::RL { r8: R8::A }.execute(core);
                core.set_z_flag(false);
                1
            }
            Instruction::RRA => {
                Instruction::RR { r8: R8::A }.execute(core);
                core.set_z_flag(false);
                1
            }
            Instruction::DAA => {
                if core.get_n_flag() {
                    let mut adj = 0;
                    if core.get_h_flag() {
                        adj += 0x6
                    }
                    if core.get_c_flag() {
                        adj += 0x60
                    }
                    core.set_a(core.get_a() - adj);
                } else {
                    let mut adj = 0;
                    let a = core.get_a();
                    if core.get_h_flag() || a & 0xF > 0x9 {
                        adj += 0x6
                    }
                    if core.get_c_flag() || a > 0x99 {
                        adj += 0x60;
                        core.set_c_flag(false);
                    }
                    core.set_a(a + adj);
                }
                core.set_z_flag(core.get_a() == 0);
                core.set_h_flag(false);
                1
            }
            Instruction::CPL => {
                core.set_a(!core.get_a());
                core.set_n_flag(true);
                core.set_h_flag(true);
                1
            }
            Instruction::SCF => {
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(true);
                1
            }
            Instruction::CCF => {
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(!core.get_c_flag());
                1
            }
            Instruction::JR { imm8 } => {
                let pc = core.get_pc() as i32;
                let offset = (*imm8 as i8) as i32;
                let new_pc = (pc + offset) as u16;
                core.set_pc(new_pc);
                3
            }
            Instruction::JRcond { imm8, cond } => {
                if cond.check(core) {
                    Instruction::JR { imm8: *imm8 }.execute(core);
                    3
                } else {
                    2
                }
            }
            // just let the core deal with this krangled instruction themself
            Instruction::STOP => unimplemented!(),
            Instruction::LDR { dest, src } => {
                dest.set(core, src.get(core));
                1
            }
            Instruction::HALT => {
                core.set_halt();
                1
            }
            Instruction::ADD { r8 } => {
                // let val = r8.get(core);
                // let a = core.get_a();
                // let (res, c) = a.overflowing_add(val);
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(false);
                // core.set_h_flag((a & 0x0F) + (val & 0x0F) > 0x0F);
                // core.set_c_flag(c);
                Instruction::ADDimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::ADC { r8 } => {
                // let val = r8.get(core);
                // let old = core.get_a();
                // let (new, c_val) = old.overflowing_add(val);
                // let c = core.get_c_flag() as u8;
                // let (res, c_c) = new.overflowing_add(c);
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(false);
                // core.set_h_flag((old & 0x0F) + (val & 0x0F) + c > 0x0F);
                // core.set_c_flag(c_val || c_c);
                Instruction::ADCimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::SUB { r8 } => {
                // let val = r8.get(core);
                // let old = core.get_a();
                // let (res, c) = old.overflowing_sub(val);
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(true);
                // core.set_h_flag((val & 0x0F) > (old & 0x0F));
                // core.set_c_flag(c);
                Instruction::SUBimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::SBC { r8 } => {
                // let val = r8.get(core);
                // let old = core.get_a();
                // let (new, c_val) = old.overflowing_sub(val);
                // let c = core.get_c_flag() as u8;
                // let (res, c_c) = new.overflowing_sub(c);
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(true);
                // core.set_h_flag(((val & 0x0F) + c) > (old & 0x0F));
                // core.set_c_flag(c_val || c_c);
                Instruction::SBCimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::AND { r8 } => {
                // let val = r8.get(core);
                // let res = core.get_a() & val;
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(false);
                // core.set_h_flag(true);
                // core.set_c_flag(false);
                Instruction::ANDimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::XOR { r8 } => {
                // let val = r8.get(core);
                // let res = core.get_a() ^ val;
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(false);
                // core.set_h_flag(false);
                // core.set_c_flag(false);
                Instruction::XORimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::OR { r8 } => {
                // let val = r8.get(core);
                // let res = core.get_a() | val;
                // core.set_a(res);
                // core.set_z_flag(res == 0);
                // core.set_n_flag(false);
                // core.set_h_flag(false);
                // core.set_c_flag(false);
                Instruction::ORimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::CP { r8 } => {
                // let val = r8.get(core);
                // let a = core.get_a();
                // core.set_z_flag(a == val);
                // core.set_n_flag(true);
                // core.set_h_flag((val & 0x0F) > (a & 0x0F));
                // core.set_c_flag(val > a);
                Instruction::CPimm { imm8: r8.get(core) }.execute(core);
                1
            }
            Instruction::ADDimm { imm8 } => {
                let val = *imm8;
                let a = core.get_a();
                let (res, c) = a.overflowing_add(val);
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag((a & 0x0F) + (val & 0x0F) > 0x0F);
                core.set_c_flag(c);
                2
            }
            Instruction::ADCimm { imm8 } => {
                let val = *imm8;
                let old = core.get_a();
                let (new, c_val) = old.overflowing_add(val);
                let c = core.get_c_flag() as u8;
                let (res, c_c) = new.overflowing_add(c);
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag((old & 0x0F) + (val & 0x0F) + c > 0x0F);
                core.set_c_flag(c_val || c_c);
                2
            }
            Instruction::SUBimm { imm8 } => {
                let val = *imm8;
                let old = core.get_a();
                let (res, c) = old.overflowing_sub(val);
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(true);
                core.set_h_flag((val & 0x0F) > (old & 0x0F));
                core.set_c_flag(c);
                2
            }
            Instruction::SBCimm { imm8 } => {
                let val = *imm8;
                let old = core.get_a();
                let (new, c_val) = old.overflowing_sub(val);
                let c = core.get_c_flag() as u8;
                let (res, c_c) = new.overflowing_sub(c);
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(true);
                core.set_h_flag(((val & 0x0F) + c) > (old & 0x0F));
                core.set_c_flag(c_val || c_c);
                2
            }
            Instruction::ANDimm { imm8 } => {
                let val = *imm8;
                let res = core.get_a() & val;
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(true);
                core.set_c_flag(false);
                2
            }
            Instruction::XORimm { imm8 } => {
                let val = *imm8;
                let res = core.get_a() ^ val;
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(false);
                2
            }
            Instruction::ORimm { imm8 } => {
                let val = *imm8;
                let res = core.get_a() | val;
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(false);
                2
            }
            Instruction::CPimm { imm8 } => {
                let val = *imm8;
                let a = core.get_a();
                core.set_z_flag(a == val);
                core.set_n_flag(true);
                core.set_h_flag((val & 0x0F) > (a & 0x0F));
                core.set_c_flag(val > a);
                2
            }
            Instruction::RETcond { cond } => {
                if cond.check(core) {
                    Instruction::RET.execute(core);
                    5
                } else {
                    2
                }
            }
            Instruction::RET => {
                let mut sp = core.get_sp();
                let low = core.read(sp);
                sp += 1;
                let high = core.read(sp + 1);
                sp += 1;
                core.set_sp(sp);
                let val = ((high as u16) << 8) + low as u16;
                core.set_pc(val);
                4
            }
            Instruction::RETI => {
                core.ei_instantly();
                Instruction::RET.execute(core);
                4
            }
            Instruction::JPcond { cond, imm16 } => {
                if cond.check(core) {
                    Instruction::JPimm { imm16: *imm16 }.execute(core);
                    4
                } else {
                    3
                }
            }
            Instruction::JPimm { imm16 } => {
                core.set_pc(*imm16);
                4
            }
            Instruction::JPhl => {
                core.set_pc(core.get_hl());
                1
            }
            Instruction::CALLcond { cond, imm16 } => {
                if cond.check(core) {
                    Instruction::CALLimm { imm16: *imm16 }.execute(core);
                    6
                } else {
                    3
                }
            }
            Instruction::CALLimm { imm16 } => {
                let val = core.get_pc();
                let low = val as u8;
                let high = (val >> 8) as u8;
                let mut sp = core.get_sp();
                sp -= 1;
                core.write(sp, high);
                sp -= 1;
                core.write(sp, low);
                core.set_sp(sp);
                Instruction::JPimm { imm16: *imm16 }.execute(core);
                6
            }
            Instruction::RST { tgt3 } => {
                Instruction::CALLimm {
                    imm16: *tgt3 as u16 * 8,
                }
                .execute(core);
                4
            }
            Instruction::POP { r16stk } => {
                let mut sp = core.get_sp();
                let low = core.read(sp);
                sp += 1;
                let high = core.read(sp);
                sp += 1;
                core.set_sp(sp);
                let val = ((high as u16) << 8) + low as u16;
                r16stk.set(core, val);
                3
            }
            Instruction::PUSH { r16stk } => {
                let mut sp = core.get_sp();
                let val = r16stk.get(core);
                let low = val as u8;
                let high = (val >> 8) as u8;
                sp -= 1;
                core.write(sp, high);
                sp -= 1;
                core.write(sp, low);
                core.set_sp(sp);
                4
            }
            Instruction::RLC { r8 } => {
                let val = r8.get(core);
                let res = val.rotate_left(1);
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(res & 1 == 1);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::RRC { r8 } => {
                let val = r8.get(core);
                let res = val.rotate_right(1);
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(val & 1 == 1);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::RL { r8 } => {
                let val = r8.get(core);
                let rot = val.rotate_left(1);
                let res = rot & 0xFE | core.get_c_flag() as u8;
                let c = rot & 1 == 1;
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(c);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::RR { r8 } => {
                let val = r8.get(core);
                let c = val & 1 == 1;
                let flp = val & 0xFE | core.get_c_flag() as u8;
                let res = flp.rotate_left(1);
                core.set_a(res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(c);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::SLA { r8 } => {
                let val = r8.get(core);
                let c = val & 0x80 != 0;
                let res = val << 1;
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(c);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::SRA { r8 } => {
                let val = r8.get(core);
                let c = val & 1 != 0;
                let b8 = val & 0x80;
                let res = (val >> 1) | b8;
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(c);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::SWAP { r8 } => {
                let val = r8.get(core);
                let low = val & 0x0F;
                let high = val & 0xF0;
                let res = (low << 4) | (high >> 4);
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(false);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::SRL { r8 } => {
                let val = r8.get(core);
                let c = val & 1 != 0;
                let res = val >> 1;
                r8.set(core, res);
                core.set_z_flag(res == 0);
                core.set_n_flag(false);
                core.set_h_flag(false);
                core.set_c_flag(c);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::BIT { b3, r8 } => {
                let val = r8.get(core);
                let z = (val & (1 << b3)) == 0;
                core.set_z_flag(z);
                core.set_n_flag(false);
                core.set_h_flag(true);
                if matches!(r8, R8::HLmem) { 3 } else { 2 }
            }
            Instruction::RES { b3, r8 } => {
                let val = r8.get(core);
                let mask = !(1 << b3);
                let res = val & mask;
                r8.set(core, res);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::SET { b3, r8 } => {
                let val = r8.get(core);
                let mask = 1 << b3;
                let res = val | mask;
                r8.set(core, res);
                if matches!(r8, R8::HLmem) { 4 } else { 2 }
            }
            Instruction::STH => {
                let c = core.get_c() as u16;
                let addr = 0xFF00 + c;
                core.write(addr, core.get_a());
                2
            }
            Instruction::STHaddr { imm8 } => {
                let addr = 0xFF00 + (*imm8 as u16);
                core.write(addr, core.get_a());
                3
            }
            Instruction::STRaddr { imm16 } => {
                core.write(*imm16, core.get_a());
                4
            }
            Instruction::LDH => {
                let c = core.get_c() as u16;
                let addr = 0xFF00 + c;
                let val = core.read(addr);
                core.set_a(val);
                2
            }
            Instruction::LDHaddr { imm8 } => {
                let addr = 0xFF00 + (*imm8 as u16);
                let val = core.read(addr);
                core.set_a(val);
                3
            }
            Instruction::LDaddr { imm16 } => {
                let val = core.read(*imm16);
                core.set_a(val);
                4
            }
            Instruction::ADDsp { imm8 } => {
                let val = *imm8 as i8;
                let sp = core.get_sp();
                let res = ((sp as i32) + (val as i32)) as u16;
                core.set_sp(res);
                core.set_z_flag(false);
                core.set_n_flag(false);
                core.set_h_flag((res & 0xFFF0) > (sp & 0xFFF0));
                core.set_c_flag((res & 0xFF00) > (sp & 0xFF00));
                4
            }
            Instruction::LDhl { imm8 } => {
                Instruction::ADDsp { imm8: *imm8 }.execute(core);
                core.set_hl(core.get_sp());
                3
            }
            Instruction::LDsp => {
                core.set_hl(core.get_sp());
                2
            }
            Instruction::DI => {
                core.di();
                1
            }
            Instruction::EI => {
                core.ei();
                1
            }
        }
    }
}

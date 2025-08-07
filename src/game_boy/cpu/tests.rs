use crate::game_boy::context::interrupts::{Interrupt, InterruptFlags};

use super::{
    CPUState, Cpu, CpuContext,
    opcode::{Condition, Opcode, R8},
};

struct StubContext {
    pub cycle_count: usize,
    pub read_value: u8,
}

impl StubContext {
    fn with_read_value(val: u8) -> Self {
        Self {
            cycle_count: 0,
            read_value: val,
        }
    }
}

impl CpuContext for StubContext {
    fn cycle_read_itrs(&mut self, _: u16) -> (u8, InterruptFlags) {
        self.cycle();
        (self.read_value, InterruptFlags::new())
    }

    fn cycle_write_itrs(&mut self, _addr: u16, _data: u8) -> InterruptFlags {
        self.cycle();
        InterruptFlags::new()
    }

    fn cycle_state_itrs(&mut self, _state: CPUState) -> InterruptFlags {
        self.cycle_count += 1;
        InterruptFlags::new()
    }

    fn ack_interrupt(&mut self, _: Interrupt) {}

    fn has_interrupt(&mut self) -> bool {
        false
    }

    fn speed_switch(&mut self) {}

    fn has_pressed_input(&self) -> bool {
        false
    }
}
#[test]
fn instruction_duration() {
    for i in 0..255u8 {
        let opcode = Opcode::lookup(i);
        if matches!(opcode, Opcode::STOP | Opcode::INVALID) {
            // TODO: remove when instructions are implemented
            continue;
        }
        let mut cpu = Cpu::default();
        let mut context = StubContext::with_read_value(i);

        // Load opcode
        cpu.step(&mut context);

        // Execute
        context.cycle_count = 0;
        cpu.step(&mut context);

        let instruction_duration = match opcode {
            Opcode::NOP => 1,
            Opcode::LD_r16_imm16 { dest: _ } => 3,
            Opcode::LD_r16mem_a { r16mem: _ } => 2,
            Opcode::LD_a_r16mem { r16mem: _ } => 2,
            Opcode::LD_imm16_sp => 5,
            Opcode::INC_r16 { r16: _ } => 2,
            Opcode::DEC_r16 { r16: _ } => 2,
            Opcode::ADD_hl_r16 { r16: _ } => 2,
            Opcode::INC_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    3
                } else {
                    1
                }
            }
            Opcode::DEC_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    3
                } else {
                    1
                }
            }
            Opcode::LD_r8_imm8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    3
                } else {
                    2
                }
            }
            Opcode::RLCA => 1,
            Opcode::RRCA => 1,
            Opcode::RLA => 1,
            Opcode::RRA => 1,
            Opcode::DAA => 1,
            Opcode::CPL => 1,
            Opcode::SCF => 1,
            Opcode::CCF => 1,
            Opcode::JR_imm8 => 2,
            Opcode::JR_cond_imm8 { cond } => {
                if matches!(cond, Condition::NZ | Condition::NC) {
                    3
                } else {
                    2
                }
            }
            Opcode::STOP => 2,
            Opcode::LD_r8_r8 { dest, src } => {
                if dest == src {
                    1
                } else if matches!(dest, R8::HLaddr) || matches!(src, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::HALT => 1,
            Opcode::ADD_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::ADC_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::SUB_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::SBC_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::AND_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::XOR_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::OR_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::CP_a_r8 { r8 } => {
                if matches!(r8, R8::HLaddr) {
                    2
                } else {
                    1
                }
            }
            Opcode::ADD_a_imm8 => 2,
            Opcode::ADC_a_imm8 => 2,
            Opcode::SUB_a_imm8 => 2,
            Opcode::SBC_a_imm8 => 2,
            Opcode::AND_a_imm8 => 2,
            Opcode::XOR_a_imm8 => 2,
            Opcode::OR_a_imm8 => 2,
            Opcode::CP_a_imm8 => 2,
            Opcode::RET_cond { cond } => {
                if matches!(cond, Condition::NZ | Condition::NC) {
                    5
                } else {
                    2
                }
            }
            Opcode::RET => 4,
            Opcode::RETI => 4,
            Opcode::JP_cond_imm16 { cond } => {
                if matches!(cond, Condition::NZ | Condition::NC) {
                    4
                } else {
                    3
                }
            }
            Opcode::JP_imm16 => 4,
            Opcode::JP_hl => 1,
            Opcode::CALL_cond_imm16 { cond } => {
                if matches!(cond, Condition::NZ | Condition::NC) {
                    6
                } else {
                    3
                }
            }
            Opcode::CALL_imm16 => 6,
            Opcode::RST { tgt3: _ } => 4,
            Opcode::POP { r16stk: _ } => 3,
            Opcode::PUSH { r16stk: _ } => 4,
            Opcode::PREFIX => 2,
            Opcode::LDH_c_a => 2,
            Opcode::LDH_imm8_a => 3,
            Opcode::LD_imm16_a => 4,
            Opcode::LDH_a_c => 2,
            Opcode::LDH_a_imm8 => 3,
            Opcode::LD_a_imm16 => 4,
            Opcode::ADD_sp_imm8 => 4,
            Opcode::LD_hl_spimm8 => 3,
            Opcode::LD_sp_hl => 2,
            Opcode::DI => 1,
            Opcode::EI => 1,
            Opcode::INVALID => 1,
        };
        if matches!(opcode, Opcode::INVALID) {
            // TODO: test invalid event
            continue;
        }
        assert_eq!(
            context.cycle_count, instruction_duration,
            "testing duration of instruction {opcode}"
        );
    }
}

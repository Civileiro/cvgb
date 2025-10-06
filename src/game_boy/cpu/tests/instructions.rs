use crate::game_boy::{
    Opcode,
    cpu::{
        CPUState,
        opcode::{Condition, R8, R16mem},
        registers::{Flags, Reg8, Reg16},
        tests::{
            END_INSTRUCTION,
            macros::{
                arith_cycles, arith_mem, set_reg, test_adc_a, test_add_a, test_add_hl, test_and_a,
                test_arithmetic, test_call_cc, test_cp_a, test_dec_r8, test_dec_r16, test_decode,
                test_inc_r8, test_inc_r16, test_invalid, test_jp_cc, test_jr_cc, test_ld_a_r16mem,
                test_ld_r8_imm, test_ld_r8_r8, test_ld_r16_imm, test_ld_r16mem_a, test_or_a,
                test_pop, test_push, test_ret_cc, test_rst, test_sbc_a, test_sub_a, test_xor_a,
            },
            run_test,
        },
    },
};

#[test]
fn test_0x00() {
    let (cpu, ctx) = run_test(&[0x00], |cpu| {
        assert_eq!(cpu.opcode, Opcode::NOP);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.f, Flags::new());
}

test_ld_r16_imm!(0x01, b, c, Opcode::LD_r16_imm16 { dest: Reg16::BC });

test_ld_r16mem_a!(0x02, b, c, Opcode::LD_r16mem_a { r16mem: R16mem::BC });

test_inc_r16!(0x03, b, c, Opcode::INC_r16 { r16: Reg16::BC });

test_inc_r8!(
    0x04,
    b,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_dec_r8!(
    0x05,
    b,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_ld_r8_imm!(
    0x06,
    b,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::B)
    }
);

#[test]
fn test_0x07() {
    let (cpu, ctx) = run_test(&[0x07], |cpu| {
        assert_eq!(cpu.opcode, Opcode::RLCA);
        cpu.regs.a = 0x79;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xF2);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x07_carry() {
    let (cpu, ctx) = run_test(&[0x07], |cpu| {
        cpu.regs.a = 0xF0;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xE1);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x08() {
    let (cpu, ctx) = run_test(&[0x08, 0xEF, 0xBE], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LD_imm16_sp);
        cpu.regs.sp = 0xDEAD;
    });
    assert_eq!(ctx.cycle_count, 5);
    assert_eq!(cpu.regs.sp, 0xDEAD);
    assert_eq!(cpu.regs.f, Flags::new());
    assert_eq!(ctx.memory[0xBEEF], 0xAD);
    assert_eq!(ctx.memory[0xBEEF + 1], 0xDE);
}

test_add_hl!(0x09, b, c, Opcode::ADD_hl_r16 { r16: Reg16::BC });

test_ld_a_r16mem!(0x0a, b, c, Opcode::LD_a_r16mem { r16mem: R16mem::BC });

test_dec_r16!(0x0b, b, c, Opcode::DEC_r16 { r16: Reg16::BC });

test_inc_r8!(
    0x0c,
    c,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_dec_r8!(
    0x0d,
    c,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_ld_r8_imm!(
    0x0e,
    c,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::C)
    }
);

#[test]
fn test_0x0f() {
    let (cpu, ctx) = run_test(&[0x0f], |cpu| {
        assert_eq!(cpu.opcode, Opcode::RRCA);
        cpu.regs.a = 0x78;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0x3C);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x0f_carry() {
    let (cpu, ctx) = run_test(&[0x0f], |cpu| {
        cpu.regs.a = 0x79;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xBC);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x10() {
    let (cpu, ctx) = run_test(&[0x10, 0x00], |cpu| {
        assert_eq!(cpu.opcode, Opcode::STOP);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.state, CPUState::Stop);
}

test_ld_r16_imm!(0x11, d, e, Opcode::LD_r16_imm16 { dest: Reg16::DE });

test_ld_r16mem_a!(0x12, d, e, Opcode::LD_r16mem_a { r16mem: R16mem::DE });

test_inc_r16!(0x13, d, e, Opcode::INC_r16 { r16: Reg16::DE });

test_inc_r8!(
    0x14,
    d,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_dec_r8!(
    0x15,
    d,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_ld_r8_imm!(
    0x16,
    d,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::D)
    }
);

#[test]
fn test_0x17() {
    let (cpu, ctx) = run_test(&[0x17], |cpu| {
        assert_eq!(cpu.opcode, Opcode::RLA);
        cpu.regs.a = 0x79;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xF2);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x17_carry() {
    let (cpu, ctx) = run_test(&[0x17], |cpu| {
        cpu.regs.a = 0xF9;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xF2);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x17_carry1() {
    let (cpu, ctx) = run_test(&[0x17], |cpu| {
        cpu.regs.a = 0xF9;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
        cpu.regs.f.set_c(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xF3);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x18() {
    let (cpu, ctx) = run_test(&[0x18, 0x02, 0x00, 0x00], |cpu| {
        assert_eq!(cpu.opcode, Opcode::JR_imm8);
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(cpu.regs.pc, 0x0005);
    assert_eq!(cpu.regs.f, Flags::new());
}

test_add_hl!(0x19, d, e, Opcode::ADD_hl_r16 { r16: Reg16::DE });

test_ld_a_r16mem!(0x1a, d, e, Opcode::LD_a_r16mem { r16mem: R16mem::DE });

test_dec_r16!(0x1b, d, e, Opcode::DEC_r16 { r16: Reg16::DE });

test_inc_r8!(
    0x1c,
    e,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_dec_r8!(
    0x1d,
    e,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_ld_r8_imm!(
    0x1e,
    e,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::E)
    }
);

#[test]
fn test_0x1f() {
    let (cpu, ctx) = run_test(&[0x1f], |cpu| {
        assert_eq!(cpu.opcode, Opcode::RRA);
        cpu.regs.a = 0x78;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0x3C);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x1f_carry() {
    let (cpu, ctx) = run_test(&[0x1f], |cpu| {
        cpu.regs.a = 0x79;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0x3C);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x1f_carry1() {
    let (cpu, ctx) = run_test(&[0x1f], |cpu| {
        cpu.regs.a = 0x79;
        cpu.regs.f.set_z(true);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
        cpu.regs.f.set_c(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, 0xBC);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

test_jr_cc!(
    0x20,
    z,
    false,
    Opcode::JR_cond_imm8 {
        cond: Condition::NZ
    }
);

test_ld_r16_imm!(0x21, h, l, Opcode::LD_r16_imm16 { dest: Reg16::HL });

test_ld_r16mem_a!(
    0x22,
    h,
    l,
    Opcode::LD_r16mem_a {
        r16mem: R16mem::HLi
    }
);

test_inc_r16!(0x23, h, l, Opcode::INC_r16 { r16: Reg16::HL });

test_inc_r8!(
    0x24,
    h,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_dec_r8!(
    0x25,
    h,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_ld_r8_imm!(
    0x26,
    h,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::H)
    }
);

// not testing 0x27 DAA thats too complicated

test_jr_cc!(0x28, z, true, Opcode::JR_cond_imm8 { cond: Condition::Z });

#[test]
fn test_0x29() {
    let (cpu, ctx) = run_test(&[0x29], |cpu| {
        cpu.regs.h = 0x03;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0x06);
    assert_eq!(cpu.regs.l, 0x04);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x29_carry() {
    let (cpu, ctx) = run_test(&[0x29], |cpu| {
        cpu.regs.h = 0x80;
        cpu.regs.l = 0x0F;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0x00);
    assert_eq!(cpu.regs.l, 0x1E);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x29_halfcarry() {
    let (cpu, ctx) = run_test(&[0x29], |cpu| {
        cpu.regs.h = 0x0F;
        cpu.regs.l = 0xFF;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0x1F);
    assert_eq!(cpu.regs.l, 0xFE);
    assert_eq!(cpu.regs.f, Flags::new().with_h(true));
}

#[test]
fn test_0x29_carry_halfcarry() {
    let (cpu, ctx) = run_test(&[0x29], |cpu| {
        cpu.regs.h = 0xFF;
        cpu.regs.l = 0xFF;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0xFF);
    assert_eq!(cpu.regs.l, 0xFE);
    assert_eq!(cpu.regs.f, Flags::new().with_h(true).with_c(true));
}

test_ld_a_r16mem!(
    0x2a,
    h,
    l,
    Opcode::LD_a_r16mem {
        r16mem: R16mem::HLi
    }
);

test_dec_r16!(0x2b, h, l, Opcode::DEC_r16 { r16: Reg16::HL });

test_inc_r8!(
    0x2c,
    l,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_dec_r8!(
    0x2d,
    l,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_ld_r8_imm!(
    0x2e,
    l,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::L)
    }
);

#[test]
fn test_0x2f() {
    let (cpu, ctx) = run_test(&[0x2f], |cpu| {
        assert_eq!(cpu.opcode, Opcode::CPL);
        cpu.regs.a = 0x79;
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.a, !0x79);
    assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_h(true));
}

test_jr_cc!(
    0x30,
    c,
    false,
    Opcode::JR_cond_imm8 {
        cond: Condition::NC
    }
);

#[test]
fn test_0x31() {
    let (cpu, ctx) = run_test(&[0x31, 0xEF, 0xBE], |_| {});
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(cpu.regs.sp, 0xBEEF);
    assert_eq!(cpu.regs.f, Flags::new());
}

test_ld_r16mem_a!(
    0x32,
    h,
    l,
    Opcode::LD_r16mem_a {
        r16mem: R16mem::HLd
    }
);

#[test]
fn test_0x33() {
    let (cpu, ctx) = run_test(&[0x33], |cpu| {
        assert_eq!(cpu.opcode, Opcode::INC_r16 { r16: Reg16::SP });
        cpu.regs.sp = 0xBEEF;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.sp, 0xBEF0);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x34() {
    let (cpu, ctx) = run_test(&[0x34, END_INSTRUCTION, 0x00], |cpu| {
        assert_eq!(cpu.opcode, Opcode::INC_r8 { r8: R8::HLaddr });
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0x01);
    assert_eq!(cpu.regs.f, Flags::new())
}

#[test]
fn test_0x34_overflow() {
    let (cpu, ctx) = run_test(&[0x34, END_INSTRUCTION, 0xFF], |cpu| {
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0x00);
    assert_eq!(cpu.regs.f, Flags::new().with_z(true).with_h(true));
}

#[test]
fn test_0x34_halfcarry() {
    let (cpu, ctx) = run_test(&[0x34, END_INSTRUCTION, 0x0F], |cpu| {
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0x10);
    assert_eq!(cpu.regs.f, Flags::new().with_h(true));
}

#[test]
fn test_0x35() {
    let (cpu, ctx) = run_test(&[0x35, END_INSTRUCTION, 0x79], |cpu| {
        assert_eq!(cpu.opcode, Opcode::DEC_r8 { r8: R8::HLaddr });
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0x78);
    assert_eq!(cpu.regs.f, Flags::new().with_n(true))
}

#[test]
fn test_0x35_zero() {
    let (cpu, ctx) = run_test(&[0x35, END_INSTRUCTION, 0x01], |cpu| {
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0x00);
    assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_z(true));
}

#[test]
fn test_0x35_halfcarry() {
    let (cpu, ctx) = run_test(&[0x35, END_INSTRUCTION, 0x10], |cpu| {
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0x0F);
    assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_h(true));
}

#[test]
fn test_0x35_halfcarry0() {
    let (cpu, ctx) = run_test(&[0x35, END_INSTRUCTION, 0x00], |cpu| {
        cpu.regs.h = 0x00;
        cpu.regs.l = 0x02;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0x0002], 0xFF);
    assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_h(true));
}

#[test]
fn test_0x36() {
    let (cpu, ctx) = run_test(&[0x36, 0x79], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LD_r8_imm8 { r8: R8::HLaddr });
        cpu.regs.h = 0xBE;
        cpu.regs.l = 0xEF;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0xBEEF], 0x79);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x37() {
    let (cpu, ctx) = run_test(&[0x37], |cpu| {
        assert_eq!(cpu.opcode, Opcode::SCF);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

test_jr_cc!(0x38, c, true, Opcode::JR_cond_imm8 { cond: Condition::C });

#[test]
fn test_0x39() {
    let (cpu, ctx) = run_test(&[0x39], |cpu| {
        cpu.regs.h = 0xF0;
        cpu.regs.l = 0xF0;
        cpu.regs.sp = 0x0F0F
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0xFF);
    assert_eq!(cpu.regs.l, 0xFF);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x39_carry() {
    let (cpu, ctx) = run_test(&[0x39], |cpu| {
        cpu.regs.h = 0xF0;
        cpu.regs.l = 0xF0;
        cpu.regs.sp = 0xF000;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0xE0);
    assert_eq!(cpu.regs.l, 0xF0);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x39_halfcarry() {
    let (cpu, ctx) = run_test(&[0x39], |cpu| {
        cpu.regs.h = 0x0F;
        cpu.regs.l = 0xF0;
        cpu.regs.sp = 0x0010
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0x10);
    assert_eq!(cpu.regs.l, 0x00);
    assert_eq!(cpu.regs.f, Flags::new().with_h(true));
}

#[test]
fn test_0x39_carry_halfcarry() {
    let (cpu, ctx) = run_test(&[0x39], |cpu| {
        cpu.regs.h = 0xFF;
        cpu.regs.l = 0xF0;
        cpu.regs.sp = 0x0010;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.h, 0x00);
    assert_eq!(cpu.regs.l, 0x00);
    assert_eq!(cpu.regs.f, Flags::new().with_h(true).with_c(true));
}

test_ld_a_r16mem!(
    0x3a,
    h,
    l,
    Opcode::LD_a_r16mem {
        r16mem: R16mem::HLd
    }
);

#[test]
fn test_0x3b() {
    let (cpu, ctx) = run_test(&[0x3b], |cpu| {
        assert_eq!(cpu.opcode, Opcode::DEC_r16 { r16: Reg16::SP });
        cpu.regs.sp = 0x0F00;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.sp, 0x0EFF);
    assert_eq!(cpu.regs.f, Flags::new());
}

test_inc_r8!(
    0x3c,
    a,
    Opcode::INC_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_dec_r8!(
    0x3d,
    a,
    Opcode::DEC_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_ld_r8_imm!(
    0x3e,
    a,
    Opcode::LD_r8_imm8 {
        r8: R8::Reg(Reg8::A)
    }
);

#[test]
fn test_0x3f() {
    let (cpu, ctx) = run_test(&[0x3f], |cpu| {
        assert_eq!(cpu.opcode, Opcode::CCF);
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0x3f_true() {
    let (cpu, ctx) = run_test(&[0x3f], |cpu| {
        cpu.regs.f.set_n(true);
        cpu.regs.f.set_h(true);
        cpu.regs.f.set_c(true);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.regs.f, Flags::new());
}

test_ld_r8_r8!(
    0x40,
    b,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x41,
    b,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x42,
    b,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x43,
    b,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x44,
    b,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x45,
    b,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x46,
    b,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::HLaddr,
    }
);

test_ld_r8_r8!(
    0x47,
    b,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::B),
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x48,
    c,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x49,
    c,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x4a,
    c,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x4b,
    c,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x4c,
    c,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x4d,
    c,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x4e,
    c,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::HLaddr
    }
);

test_ld_r8_r8!(
    0x4f,
    c,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::C),
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x50,
    d,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x51,
    d,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x52,
    d,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x53,
    d,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x54,
    d,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x55,
    d,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x56,
    d,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::HLaddr,
    }
);

test_ld_r8_r8!(
    0x57,
    d,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::D),
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x58,
    e,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x59,
    e,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x5a,
    e,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x5b,
    e,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x5c,
    e,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x5d,
    e,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x5e,
    e,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::HLaddr
    }
);

test_ld_r8_r8!(
    0x5f,
    e,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::E),
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x60,
    h,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x61,
    h,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x62,
    h,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x63,
    h,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x64,
    h,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x65,
    h,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x66,
    h,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::HLaddr,
    }
);

test_ld_r8_r8!(
    0x67,
    h,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::H),
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x68,
    l,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x69,
    l,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x6a,
    l,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x6b,
    l,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x6c,
    l,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x6d,
    l,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x6e,
    l,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::HLaddr
    }
);

test_ld_r8_r8!(
    0x6f,
    l,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::L),
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x70,
    hl,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::HLaddr,
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x71,
    hl,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::HLaddr,
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x72,
    hl,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::HLaddr,
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x73,
    hl,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::HLaddr,
        src: R8::Reg(Reg8::E)
    }
);

#[test]
fn test_0x74() {
    let (cpu, ctx) = run_test(&[0x74], |cpu| {
        assert_eq!(
            cpu.opcode,
            Opcode::LD_r8_r8 {
                dest: R8::HLaddr,
                src: R8::Reg(Reg8::H)
            }
        );
        cpu.regs.h = 0xBE;
        cpu.regs.l = 0xEF;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(ctx.memory[0xBEEF], 0xBE);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x75() {
    let (cpu, ctx) = run_test(&[0x75], |cpu| {
        assert_eq!(
            cpu.opcode,
            Opcode::LD_r8_r8 {
                dest: R8::HLaddr,
                src: R8::Reg(Reg8::L)
            }
        );
        cpu.regs.h = 0xBE;
        cpu.regs.l = 0xEF;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(ctx.memory[0xBEEF], 0xEF);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0x76() {
    let (cpu, ctx) = run_test(&[0x76], |cpu| {
        assert_eq!(cpu.opcode, Opcode::HALT);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert_eq!(cpu.state, CPUState::Halt(0));
}

test_ld_r8_r8!(
    0x77,
    hl,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::HLaddr,
        src: R8::Reg(Reg8::A)
    }
);

test_ld_r8_r8!(
    0x78,
    a,
    b,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::B)
    }
);

test_ld_r8_r8!(
    0x79,
    a,
    c,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::C)
    }
);

test_ld_r8_r8!(
    0x7a,
    a,
    d,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::D)
    }
);

test_ld_r8_r8!(
    0x7b,
    a,
    e,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::E)
    }
);

test_ld_r8_r8!(
    0x7c,
    a,
    h,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::H)
    }
);

test_ld_r8_r8!(
    0x7d,
    a,
    l,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::L)
    }
);

test_ld_r8_r8!(
    0x7e,
    a,
    hl,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::HLaddr
    }
);

test_ld_r8_r8!(
    0x7f,
    a,
    a,
    Opcode::LD_r8_r8 {
        dest: R8::Reg(Reg8::A),
        src: R8::Reg(Reg8::A)
    }
);

test_add_a!(
    0x80,
    b,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_add_a!(
    0x81,
    c,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_add_a!(
    0x82,
    d,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_add_a!(
    0x83,
    e,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_add_a!(
    0x84,
    h,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_add_a!(
    0x85,
    l,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_add_a!(0x86, hl, Opcode::ADD_a_r8 { r8: R8::HLaddr });

test_add_a!(
    0x87,
    a,
    Opcode::ADD_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_adc_a!(
    0x88,
    b,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_adc_a!(
    0x89,
    c,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_adc_a!(
    0x8a,
    d,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_adc_a!(
    0x8b,
    e,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_adc_a!(
    0x8c,
    h,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_adc_a!(
    0x8d,
    l,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_adc_a!(0x8e, hl, Opcode::ADC_a_r8 { r8: R8::HLaddr });

test_adc_a!(
    0x8f,
    a,
    Opcode::ADC_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_sub_a!(
    0x90,
    b,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_sub_a!(
    0x91,
    c,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_sub_a!(
    0x92,
    d,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_sub_a!(
    0x93,
    e,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_sub_a!(
    0x94,
    h,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_sub_a!(
    0x95,
    l,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_sub_a!(0x96, hl, Opcode::SUB_a_r8 { r8: R8::HLaddr });

test_sub_a!(
    0x97,
    a,
    Opcode::SUB_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_sbc_a!(
    0x98,
    b,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_sbc_a!(
    0x99,
    c,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_sbc_a!(
    0x9a,
    d,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_sbc_a!(
    0x9b,
    e,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_sbc_a!(
    0x9c,
    h,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_sbc_a!(
    0x9d,
    l,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_sbc_a!(0x9e, hl, Opcode::SBC_a_r8 { r8: R8::HLaddr });

test_sbc_a!(
    0x9f,
    a,
    Opcode::SBC_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_and_a!(
    0xa0,
    b,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_and_a!(
    0xa1,
    c,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_and_a!(
    0xa2,
    d,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_and_a!(
    0xa3,
    e,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_and_a!(
    0xa4,
    h,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_and_a!(
    0xa5,
    l,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_and_a!(0xa6, hl, Opcode::AND_a_r8 { r8: R8::HLaddr });

test_and_a!(
    0xa7,
    a,
    Opcode::AND_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_xor_a!(
    0xa8,
    b,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_xor_a!(
    0xa9,
    c,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_xor_a!(
    0xaa,
    d,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_xor_a!(
    0xab,
    e,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_xor_a!(
    0xac,
    h,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_xor_a!(
    0xad,
    l,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_xor_a!(0xae, hl, Opcode::XOR_a_r8 { r8: R8::HLaddr });

test_xor_a!(
    0xaf,
    a,
    Opcode::XOR_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_or_a!(
    0xb0,
    b,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_or_a!(
    0xb1,
    c,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_or_a!(
    0xb2,
    d,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_or_a!(
    0xb3,
    e,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_or_a!(
    0xb4,
    h,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_or_a!(
    0xb5,
    l,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_or_a!(0xb6, hl, Opcode::OR_a_r8 { r8: R8::HLaddr });

test_or_a!(
    0xb7,
    a,
    Opcode::OR_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_cp_a!(
    0xb8,
    b,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::B)
    }
);

test_cp_a!(
    0xb9,
    c,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::C)
    }
);

test_cp_a!(
    0xba,
    d,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::D)
    }
);

test_cp_a!(
    0xbb,
    e,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::E)
    }
);

test_cp_a!(
    0xbc,
    h,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::H)
    }
);

test_cp_a!(
    0xbd,
    l,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::L)
    }
);

test_cp_a!(0xbe, hl, Opcode::CP_a_r8 { r8: R8::HLaddr });

test_cp_a!(
    0xbf,
    a,
    Opcode::CP_a_r8 {
        r8: R8::Reg(Reg8::A)
    }
);

test_ret_cc!(
    0xc0,
    z,
    false,
    Opcode::RET_cond {
        cond: Condition::NZ
    }
);

test_pop!(0xc1, b, c, Opcode::POP { r16stk: Reg16::BC });

test_jp_cc!(
    0xc2,
    z,
    false,
    Opcode::JP_cond_imm16 {
        cond: Condition::NZ
    }
);

#[test]
fn test_0xc3() {
    let (_cpu, ctx) = run_test(&[0xc3, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00], |cpu| {
        assert_eq!(cpu.opcode, Opcode::JP_imm16)
    });
    assert_eq!(ctx.cycle_count, 4);
}

test_call_cc!(
    0xc4,
    z,
    false,
    Opcode::CALL_cond_imm16 {
        cond: Condition::NZ
    }
);

test_push!(0xc5, b, c, Opcode::PUSH { r16stk: Reg16::BC });

test_add_a!(0xc6, imm, Opcode::ADD_a_imm8);

test_rst!(0xc7, 0b000);

test_ret_cc!(0xc8, z, true, Opcode::RET_cond { cond: Condition::Z });

#[test]
fn test_0xc9() {
    let (cpu, ctx) = run_test(
        &[0xc9, END_INSTRUCTION, 0x00, 0x00, 0x07, 0x00, 0x00],
        |cpu| {
            cpu.regs.sp = 0x0004;
        },
    );
    assert_eq!(ctx.cycle_count, 4);
    assert!(!cpu.ime);
}

test_jp_cc!(0xca, z, true, Opcode::JP_cond_imm16 { cond: Condition::Z });

// 0xCB PREFIX tests in the other file

test_call_cc!(
    0xcc,
    z,
    true,
    Opcode::CALL_cond_imm16 { cond: Condition::Z }
);

#[test]
fn test_0xcd() {
    let (cpu, ctx) = run_test(
        &[0xcd, 0x07, 0x00, END_INSTRUCTION, 0x00, 0x00, 0x00],
        |cpu| {
            assert_eq!(cpu.opcode, Opcode::CALL_imm16);
            cpu.regs.sp = 0xBEEF
        },
    );
    assert_eq!(ctx.cycle_count, 6);
    assert_eq!(cpu.regs.pc, 0x0008);
    assert_eq!(cpu.regs.sp, 0xBEED);
    assert_eq!(ctx.memory[0xBEEE], 0x00);
    assert_eq!(ctx.memory[0xBEED], 0x03);
}

test_adc_a!(0xce, imm, Opcode::ADC_a_imm8);

test_rst!(0xcf, 0b001);

test_ret_cc!(
    0xd0,
    c,
    false,
    Opcode::RET_cond {
        cond: Condition::NC
    }
);

test_pop!(0xd1, d, e, Opcode::POP { r16stk: Reg16::DE });

test_jp_cc!(
    0xd2,
    c,
    false,
    Opcode::JP_cond_imm16 {
        cond: Condition::NC
    }
);

test_invalid!(0xd3);

test_call_cc!(
    0xd4,
    c,
    false,
    Opcode::CALL_cond_imm16 {
        cond: Condition::NC
    }
);

test_push!(0xd5, d, e, Opcode::PUSH { r16stk: Reg16::DE });

test_sub_a!(0xd6, imm, Opcode::SUB_a_imm8);

test_rst!(0xd7, 0b010);

test_ret_cc!(0xd8, c, true, Opcode::RET_cond { cond: Condition::C });

#[test]
fn test_0xd9() {
    let (cpu, ctx) = run_test(
        &[0xd9, END_INSTRUCTION, 0x00, 0x00, 0x07, 0x00, 0x00],
        |cpu| {
            assert_eq!(cpu.opcode, Opcode::RETI);
            cpu.regs.sp = 0x0004;
        },
    );
    assert_eq!(ctx.cycle_count, 4);
    assert!(cpu.ime);
}

test_jp_cc!(0xda, c, true, Opcode::JP_cond_imm16 { cond: Condition::C });

test_invalid!(0xdb);

test_call_cc!(
    0xdc,
    c,
    true,
    Opcode::CALL_cond_imm16 { cond: Condition::C }
);

test_invalid!(0xdd);

test_sbc_a!(0xde, imm, Opcode::SBC_a_imm8);

test_rst!(0xdf, 0b011);

#[test]
fn test_0xe0() {
    let (_cpu, ctx) = run_test(&[0xe0, 0x79], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LDH_imm8_a);
        cpu.regs.a = 0x42;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(ctx.memory[0xFF79], 0x42);
}

test_pop!(0xe1, h, l, Opcode::POP { r16stk: Reg16::HL });

#[test]
fn test_0xe2() {
    let (_cpu, ctx) = run_test(&[0xe2], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LDH_c_a);
        cpu.regs.a = 0x42;
        cpu.regs.c = 0x79;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(ctx.memory[0xFF79], 0x42);
}

test_invalid!(0xe3);

test_invalid!(0xe4);

test_push!(0xe5, h, l, Opcode::PUSH { r16stk: Reg16::HL });

test_and_a!(0xe6, imm, Opcode::AND_a_imm8);

test_rst!(0xe7, 0b100);

#[test]
fn test_0xe8() {
    let (cpu, ctx) = run_test(&[0xe8, 0x79], |cpu| {
        assert_eq!(cpu.opcode, Opcode::ADD_sp_imm8);
        cpu.regs.sp = 0x1111;
    });
    assert_eq!(ctx.cycle_count, 4);
    assert_eq!(cpu.regs.sp, 0x118a);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0xe8_carry() {
    let (cpu, ctx) = run_test(&[0xe8, 0x79], |cpu| {
        cpu.regs.sp = 0x1191;
    });
    assert_eq!(ctx.cycle_count, 4);
    assert_eq!(cpu.regs.sp, 0x120a);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0xe8_halfcarry() {
    let (cpu, ctx) = run_test(&[0xe8, 0x79], |cpu| {
        cpu.regs.sp = 0x1198;
    });
    assert_eq!(ctx.cycle_count, 4);
    assert_eq!(cpu.regs.sp, 0x1211);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true).with_h(true));
}

#[test]
fn test_0xe9() {
    let (_cpu, ctx) = run_test(&[0xe9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], |cpu| {
        assert_eq!(cpu.opcode, Opcode::JP_hl);
        cpu.regs.l = 0x07;
    });
    assert_eq!(ctx.cycle_count, 1);
}

#[test]
fn test_0xea() {
    let (_cpu, ctx) = run_test(&[0xea, 0xEF, 0xBE], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LD_imm16_a);
        cpu.regs.a = 0x79;
    });
    assert_eq!(ctx.cycle_count, 4);
    assert_eq!(ctx.memory[0xBEEF], 0x79);
}

test_invalid!(0xeb);

test_invalid!(0xec);

test_invalid!(0xed);

test_xor_a!(0xee, imm, Opcode::XOR_a_imm8);

test_rst!(0xef, 0b101);

#[test]
fn test_0xf0() {
    let (cpu, ctx) = run_test(&[0xf0, 0x79], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LDH_a_imm8);
        cpu.regs.a = 0x42;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(cpu.regs.a, 0x00);
}

test_pop!(0xf1, a, f, Opcode::POP { r16stk: Reg16::AF });

#[test]
fn test_0xf2() {
    let (cpu, ctx) = run_test(&[0xf2], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LDH_a_c);
        cpu.regs.a = 0x42;
        cpu.regs.c = 0x79;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.a, 0x00);
}

#[test]
fn test_0xf3() {
    let (cpu, ctx) = run_test(&[0xf3], |cpu| {
        assert_eq!(cpu.opcode, Opcode::DI);
        cpu.ime = true;
    });
    assert_eq!(ctx.cycle_count, 1);
    assert!(!cpu.ime);
}

test_invalid!(0xf4);

#[test]
fn test_0xf5() {
    let (cpu, ctx) = run_test(&[0xf5], |cpu| {
        cpu.regs.sp = 0xBEEF;
        cpu.regs.a = 0x79;
        cpu.regs.f = 0x42.into();
    });
    assert_eq!(ctx.cycle_count, 4);
    assert_eq!(cpu.regs.sp, 0xBEED);
    assert_eq!(ctx.memory[0xBEEE], 0x79);
    assert_eq!(ctx.memory[0xBEED], 0x40);
}

test_or_a!(0xf6, imm, Opcode::OR_a_imm8);

test_rst!(0xf7, 0b110);

#[test]
fn test_0xf8() {
    let (cpu, ctx) = run_test(&[0xf8, 0x79], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LD_hl_spimm8);
        cpu.regs.sp = 0x1111;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(cpu.regs.h, 0x11);
    assert_eq!(cpu.regs.l, 0x8a);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0xf8_carry() {
    let (cpu, ctx) = run_test(&[0xf8, 0x79], |cpu| {
        cpu.regs.sp = 0x1191;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(cpu.regs.h, 0x12);
    assert_eq!(cpu.regs.l, 0x0a);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true));
}

#[test]
fn test_0xf8_halfcarry() {
    let (cpu, ctx) = run_test(&[0xf8, 0x79], |cpu| {
        cpu.regs.sp = 0x1198;
    });
    assert_eq!(ctx.cycle_count, 3);
    assert_eq!(cpu.regs.h, 0x12);
    assert_eq!(cpu.regs.l, 0x11);
    assert_eq!(cpu.regs.f, Flags::new().with_c(true).with_h(true));
}

#[test]
fn test_0xf9() {
    let (cpu, ctx) = run_test(&[0xf9], |cpu| {
        assert_eq!(cpu.opcode, Opcode::LD_sp_hl);
        cpu.regs.sp = 0x1111;
        cpu.regs.h = 0xBE;
        cpu.regs.l = 0xEF;
    });
    assert_eq!(ctx.cycle_count, 2);
    assert_eq!(cpu.regs.sp, 0xBEEF);
    assert_eq!(cpu.regs.h, 0xBE);
    assert_eq!(cpu.regs.l, 0xEF);
    assert_eq!(cpu.regs.f, Flags::new());
}

#[test]
fn test_0xfa() {
    let (cpu, ctx) = run_test(
        &[0xfa, 0x05, 0x00, END_INSTRUCTION, 0x00, 0x79, 0x00],
        |cpu| {
            assert_eq!(cpu.opcode, Opcode::LD_a_imm16);
            cpu.regs.a = 0x42;
        },
    );
    assert_eq!(ctx.cycle_count, 4);
    assert_eq!(cpu.regs.a, 0x79);
}

#[test]
fn test_0xfb() {
    let (cpu, ctx) = run_test(&[0xfb], |cpu| {
        assert_eq!(cpu.opcode, Opcode::EI);
    });
    assert_eq!(ctx.cycle_count, 1);
    assert!(cpu.ime);
}

test_invalid!(0xfc);

test_invalid!(0xfd);

test_cp_a!(0xfe, imm, Opcode::CP_a_imm8);

test_rst!(0xff, 0b111);

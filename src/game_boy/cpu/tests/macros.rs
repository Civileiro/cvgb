macro_rules! test_decode {
    ($byte:literal, $opcode:expr) => {
        paste::paste! {
            #[test]
            fn [<test_ $byte _decode>]() {
                let _ = run_test(&[$byte], |cpu| {
                    assert_eq!(cpu.opcode, $opcode, "CPU should decode {:02X} as '{}'", $byte, $opcode);
                    cpu.state = CPUState::Stop;
                });
            }

        }
    };
}

pub(crate) use test_decode;

macro_rules! test_ld_r16_imm {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, 0xEF, 0xBE], |_| {});
                assert_eq!(ctx.cycle_count, 3);
                assert_eq!(cpu.regs.$reg1, 0xBE);
                assert_eq!(cpu.regs.$reg2, 0xEF);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_ld_r16_imm;

macro_rules! test_ld_r16mem_a {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.a = 0x79;
                    cpu.regs.$reg1 = 0xBE;
                    cpu.regs.$reg2 = 0xEF;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(ctx.memory[0xBEEF], 0x79);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_ld_r16mem_a;

macro_rules! test_inc_r16 {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg1 = 0xBE;
                    cpu.regs.$reg2 = 0xEF;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg1, 0xBE);
                assert_eq!(cpu.regs.$reg2, 0xF0);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_inc_r16;

macro_rules! test_inc_r8 {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0x79;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0x7A);
                assert_eq!(cpu.regs.f, Flags::new())
            }

            #[test]
            fn [<test_ $byte _overflow>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0xFF;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0x00);
                assert_eq!(cpu.regs.f, Flags::new().with_z(true).with_h(true));
            }

            #[test]
            fn [<test_ $byte _halfcarry>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0x0F;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0x10);
                assert_eq!(cpu.regs.f, Flags::new().with_h(true));
            }
        }
    };
}

pub(crate) use test_inc_r8;

macro_rules! test_dec_r8 {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0x79;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0x78);
                assert_eq!(cpu.regs.f, Flags::new().with_n(true));
            }

            #[test]
            fn [<test_ $byte _zero>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0x01;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0x00);
                assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_z(true));
            }

            #[test]
            fn [<test_ $byte _halfcarry>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0x10;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0x0F);
                assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_h(true));
            }

            #[test]
            fn [<test_ $byte _halfcarry0>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg = 0x00;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg, 0xFF);
                assert_eq!(cpu.regs.f, Flags::new().with_n(true).with_h(true));
            }
        }
    };
}

pub(crate) use test_dec_r8;

macro_rules! test_ld_r8_imm {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x79], |_| {});
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg, 0x79);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_ld_r8_imm;

macro_rules! test_add_hl {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.h = 0xF0;
                    cpu.regs.l = 0x0F;
                    cpu.regs.$reg1 = 0x03;
                    cpu.regs.$reg2 = 0x05;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.h, 0xF3);
                assert_eq!(cpu.regs.l, 0x14);
                assert_eq!(cpu.regs.f, Flags::new());
            }

            #[test]
            fn [<test_ $byte _carry>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.h = 0xF0;
                    cpu.regs.l = 0x0F;
                    cpu.regs.$reg1 = 0x10;
                    cpu.regs.$reg2 = 0xF0;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.h, 0x00);
                assert_eq!(cpu.regs.l, 0xFF);
                assert_eq!(cpu.regs.f, Flags::new().with_c(true));
            }

            #[test]
            fn [<test_ $byte _halfcarry>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.h = 0x0F;
                    cpu.regs.l = 0x0F;
                    cpu.regs.$reg1 = 0x00;
                    cpu.regs.$reg2 = 0xF1;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.h, 0x10);
                assert_eq!(cpu.regs.l, 0x00);
                assert_eq!(cpu.regs.f, Flags::new().with_h(true));
            }

            #[test]
            fn [<test_ $byte _carry_halfcarry>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.h = 0xFF;
                    cpu.regs.l = 0x0F;
                    cpu.regs.$reg1 = 0x00;
                    cpu.regs.$reg2 = 0xF1;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.h, 0x00);
                assert_eq!(cpu.regs.l, 0x00);
                assert_eq!(cpu.regs.f, Flags::new().with_h(true).with_c(true));
            }
        }
    };
}

pub(crate) use test_add_hl;

macro_rules! test_ld_a_r16mem {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, END_INSTRUCTION, 0x00, 0x00, 0x79], |cpu| {
                    cpu.regs.$reg1 = 0x00;
                    cpu.regs.$reg2 = 0x04;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.a, 0x79);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_ld_a_r16mem;

macro_rules! test_dec_r16 {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg1 = 0x79;
                    cpu.regs.$reg2 = 0x79;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg1, 0x79);
                assert_eq!(cpu.regs.$reg2, 0x78);
                assert_eq!(cpu.regs.f, Flags::new());
            }

            #[test]
            fn [<test_ $byte _zero>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg1 = 0x00;
                    cpu.regs.$reg2 = 0x01;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg1, 0x00);
                assert_eq!(cpu.regs.$reg2, 0x00);
                assert_eq!(cpu.regs.f, Flags::new());
            }

            #[test]
            fn [<test_ $byte _halfcarry>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg1 = 0x01;
                    cpu.regs.$reg2 = 0x00;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg1, 0x00);
                assert_eq!(cpu.regs.$reg2, 0xFF);
                assert_eq!(cpu.regs.f, Flags::new());
            }

            #[test]
            fn [<test_ $byte _halfcarry0>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg1 = 0x00;
                    cpu.regs.$reg2 = 0x00;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg1, 0xFF);
                assert_eq!(cpu.regs.$reg2, 0xFF);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_dec_r16;

macro_rules! test_jr_cc {
    ($byte:literal, $flag:ident, $success:literal, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x02, END_INSTRUCTION, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>]($success);
                });
                assert_eq!(ctx.cycle_count, 3);
                assert_eq!(cpu.regs.pc, 0x0005);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>]($success));
            }

            #[test]
            fn [<test_ $byte _fail>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x02, END_INSTRUCTION, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>](!$success);
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.pc, 0x0003);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>](!$success));
            }
        }
    };
}

pub(crate) use test_jr_cc;

macro_rules! test_ld_r8_r8 {
    ($byte:literal, $reg1:ident, hl, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, END_INSTRUCTION, 0x79], |cpu| {
                    cpu.regs.h = 0x00;
                    cpu.regs.l = 0x02;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.$reg1, 0x79);
                // assert_eq!(ctx.memory[0x0002], 0x55);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
    ($byte:literal, hl, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg2 = 0x79;
                    cpu.regs.h = 0x00;
                    cpu.regs.l = 0x02;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(ctx.memory[0x0002], 0x79);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.$reg2 = 0x79;
                });
                assert_eq!(ctx.cycle_count, 1);
                assert_eq!(cpu.regs.$reg1, 0x79);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

pub(crate) use test_ld_r8_r8;

macro_rules! set_reg {
    ($cpu:ident, imm, $input:literal) => {};
    ($cpu:ident, hl, $input:literal) => {
        $cpu.regs.h = 0x00;
        $cpu.regs.l = 0x02;
    };
    ($cpu:ident, $reg:ident, $input:literal) => {
        $cpu.regs.$reg = $input;
    };
}

pub(crate) use set_reg;

macro_rules! arith_cycles {
    (hl) => {
        2
    };
    (imm) => {
        2
    };
    ($reg:ident) => {
        1
    };
}

pub(crate) use arith_cycles;

macro_rules! arith_mem {
    ($byte:literal, imm, $input:literal) => {
        &[$byte, $input]
    };
    ($byte:literal, hl, $input:literal) => {
        &[$byte, END_INSTRUCTION, $input]
    };
    ($byte:literal, $reg:ident, $input:literal) => {
        &[$byte]
    };
}

pub(crate) use arith_mem;

macro_rules! test_arithmetic {
    ($byte:literal,
     a = $a_val:literal
     $reg:ident =
     $($input:literal $($flag_init:ident)* -> $output:literal $($flag:ident)*)+) => {
        paste::paste! {
            $(
                #[test]
                fn [<test_ $byte _case_ $input  $($flag_init)*>]() {
                    let (cpu, ctx) = run_test(arith_mem!($byte, $reg, $input), |cpu| {
                        cpu.regs.a = $a_val;
                        set_reg!(cpu, $reg, $input);
                        cpu.regs.f $(.[<set_ $flag_init>](true))*;
                    });
                    assert_eq!(ctx.cycle_count, arith_cycles!($reg));
                    assert_eq!(cpu.regs.a, $output);
                    let expected_flags = Flags::new() $(.[<with_ $flag>](true))*;
                    assert_eq!(cpu.regs.f, expected_flags);
                }
            )*
        }
    };
}

pub(crate) use test_arithmetic;

macro_rules! test_add_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_add_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x42 -> 0x84
            0x42  c-> 0x84
            0x92 -> 0x24 c
            0x92 c -> 0x24 c
            0x48 -> 0x90 h
            0x48 c -> 0x90 h
            0x80 -> 0x00 z c
            0x80 c -> 0x00 z c
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0xBB
            0x42 c -> 0xBB
            0x92 -> 0x0B c
            0x92 c -> 0x0B c
            0x47 -> 0xC0 h
            0x47 c -> 0xC0 h
            0x87 -> 0x00 z h c
            0x87 c -> 0x00 z h c
        );
    };
}

pub(crate) use test_add_a;

macro_rules! test_adc_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_adc_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x42 -> 0x84
            0x42 c -> 0x85
            0x92 -> 0x24 c
            0x92 c -> 0x25 c
            0x48 -> 0x90 h
            0x48 c -> 0x91 h
            0x80 -> 0x00 z c
            0x80 c -> 0x01 c
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0xBB
            0x42 c -> 0xBC
            0x92 -> 0x0B c
            0x92 c -> 0x0C c
            0x47 -> 0xC0 h
            0x47 c -> 0xC1 h
            0x87 -> 0x00 z h c
            0x87 c -> 0x01 h c
            0x86 -> 0xFF
            0x86 c -> 0x00 z h c
        );
    };
}

pub(crate) use test_adc_a;

macro_rules! test_sub_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_sub_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x42 -> 0x00 z n
            0x42 c -> 0x00 z n
            0x92 -> 0x00 z n
            0x92 c -> 0x00 z n
            0x48 -> 0x00 z n
            0x48 c -> 0x00 z n
            0x80 -> 0x00 z n
            0x80 c -> 0x00 z n
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0x37 n
            0x42 c -> 0x37 n
            0x92 -> 0xE7 n c
            0x92 c -> 0xE7 n c
            0x4a -> 0x2F n h
            0x4a c -> 0x2F n h
            0x7a -> 0xFF n h c
            0x7a c -> 0xFF n h c
            0x79 -> 0x00 z n
            0x79 c -> 0x00 z n
        );
    };
}

pub(crate) use test_sub_a;

macro_rules! test_sbc_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_sbc_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x42 -> 0x00 z n
            0x42 c -> 0xFF n h c
            0x92 -> 0x00 z n
            0x92 c -> 0xFF n h c
            0x48 -> 0x00 z n
            0x48 c -> 0xFF n h c
            0x80 -> 0x00 z n
            0x80 c -> 0xFF n h c
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0x37 n
            0x42 c -> 0x36 n
            0x92 -> 0xE7 n c
            0x92 c -> 0xE6 n c
            0x4a -> 0x2F n h
            0x4a c -> 0x2E n h
            0x7a -> 0xFF n h c
            0x7a c -> 0xFE n h c
            0x79 -> 0x00 z n
            0x79 c -> 0xFF n h c
            0x78 -> 0x01 n
            0x78 c -> 0x00 z n
        );
    };
}

pub(crate) use test_sbc_a;

macro_rules! test_and_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_and_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x00 -> 0x00 z h
            0x42 -> 0x42 h
            0x92 -> 0x92 h
            0x48 -> 0x48 h
            0x80 -> 0x80 h
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0x40 h
            0x92 -> 0x10 h
            0x6a -> 0x68 h
            0x86 -> 0x00 z h
        );
    };
}

pub(crate) use test_and_a;

macro_rules! test_xor_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_xor_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x00 -> 0x00 z
            0x42 -> 0x00 z
            0x92 -> 0x00 z
            0x48 -> 0x00 z
            0x80 -> 0x00 z
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0x3B
            0x92 -> 0xEB
            0x6a -> 0x13
            0x86 -> 0xFF
            0x79 -> 0x00 z
        );
    };
}

pub(crate) use test_xor_a;

macro_rules! test_or_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_or_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x00 -> 0x00 z
            0x42 -> 0x42
            0x92 -> 0x92
            0x48 -> 0x48
            0x80 -> 0x80
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0x7B
            0x92 -> 0xFB
            0x6a -> 0x7B
            0x86 -> 0xFF
            0x79 -> 0x79
        );
    };
}

pub(crate) use test_or_a;

macro_rules! test_cp_a {
    ($byte:literal, $reg:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        test_cp_a!($byte, $reg);
    };
    ($byte:literal, a) => {
        test_arithmetic!($byte,
            a = 0x00
            a =
            0x42 -> 0x42 z n
            0x92 -> 0x92 z n
            0x48 -> 0x48 z n
            0x80 -> 0x80 z n
        );
    };
    ($byte:literal, $reg:ident) => {
        test_arithmetic!($byte,
            a = 0x79
            $reg =
            0x42 -> 0x79 n
            0x92 -> 0x79 n c
            0x4a -> 0x79 n h
            0x7a -> 0x79 n h c
            0x79 -> 0x79 z n
        );
    };
}

pub(crate) use test_cp_a;

macro_rules! test_ret_cc {
    ($byte:literal, $flag:ident, $success:literal, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, END_INSTRUCTION, 0x00, 0x00, 0x07, 0x00, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>]($success);
                    cpu.regs.sp = 0x0004;
                });
                assert_eq!(ctx.cycle_count, 5);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>]($success));
            }

            #[test]
            fn [<test_ $byte _fail>]() {
                let (cpu, ctx) = run_test(&[$byte, END_INSTRUCTION, 0x00, 0x00, 0x07, 0x00, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>](!$success);
                    cpu.regs.sp = 0x0004;
                });
                assert_eq!(ctx.cycle_count, 2);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>](!$success));
            }
        }
    };
}

pub(crate) use test_ret_cc;

macro_rules! test_pop {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, END_INSTRUCTION, 0x00, 0x00, 0xEF, 0xBE, 0x00], |cpu| {
                    cpu.regs.sp = 0x0004;
                });
                assert_eq!(ctx.cycle_count, 3);
                assert_eq!(cpu.regs.$reg1, 0xBE);
                assert_eq!(cpu.regs.$reg2, 0xEF.into());
            }
        }
    };
}

pub(crate) use test_pop;

macro_rules! test_jp_cc {
    ($byte:literal, $flag:ident, $success:literal, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x07, 0x00, END_INSTRUCTION, 0x00, 0x00, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>]($success);
                });
                assert_eq!(ctx.cycle_count, 4);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>]($success));
            }

            #[test]
            fn [<test_ $byte _fail>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x07, 0x00, END_INSTRUCTION, 0x00, 0x00, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>](!$success);
                });
                assert_eq!(ctx.cycle_count, 3);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>](!$success));
            }
        }
    };
}

pub(crate) use test_jp_cc;

macro_rules! test_call_cc {
    ($byte:literal, $flag:ident, $success:literal, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x07, 0x00, END_INSTRUCTION, 0x00, 0x00, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>]($success);
                    cpu.regs.sp = 0xBEEF
                });
                assert_eq!(ctx.cycle_count, 6);
                assert_eq!(cpu.regs.pc, 0x0008);
                assert_eq!(cpu.regs.sp, 0xBEED);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>]($success));
                assert_eq!(ctx.memory[0xBEEE], 0x00);
                assert_eq!(ctx.memory[0xBEED], 0x03);
            }

            #[test]
            fn [<test_ $byte _fail>]() {
                let (cpu, ctx) = run_test(&[$byte, 0x07, 0x00, END_INSTRUCTION, 0x00, 0x00, 0x00], |cpu| {
                    cpu.regs.f.[<set_ $flag>](!$success);
                    cpu.regs.sp = 0xBEEF
                });
                assert_eq!(ctx.cycle_count, 3);
                assert_eq!(cpu.regs.pc, 0x0004);
                assert_eq!(cpu.regs.sp, 0xBEEF);
                assert_eq!(cpu.regs.f, Flags::new().[<with_ $flag>](!$success));
                assert_eq!(ctx.memory[0xBEEE], 0x00);
                assert_eq!(ctx.memory[0xBEED], 0x00);
            }
        }
    };
}

pub(crate) use test_call_cc;

macro_rules! test_push {
    ($byte:literal, $reg1:ident, $reg2:ident, $opcode:expr) => {
        test_decode!($byte, $opcode);
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&[$byte], |cpu| {
                    cpu.regs.sp = 0xBEEF;
                    cpu.regs.$reg1 = 0x79;
                    cpu.regs.$reg2 = 0x42.into();
                });
                assert_eq!(ctx.cycle_count, 4);
                assert_eq!(cpu.regs.sp, 0xBEED);
                assert_eq!(ctx.memory[0xBEEE], 0x79);
                assert_eq!(ctx.memory[0xBEED], 0x42);
            }
        }
    };
}

pub(crate) use test_push;

macro_rules! test_rst {
    ($byte:literal, $bit:literal) => {
        test_decode!($byte, Opcode::RST { tgt3: $bit });
        paste::paste! {
            #[test]
            fn [<test_ $byte>]() {
                let (cpu, ctx) = run_test(&vec![END_INSTRUCTION; 0x100], |cpu| {
                    cpu.opcode = Opcode::RST { tgt3: $bit };
                    cpu.regs.pc = 0x7942;
                    cpu.regs.sp = 0xBEEF;
                });
                assert_eq!(ctx.cycle_count, 4);
                assert_eq!(cpu.regs.pc, $bit * 8 + 1);
                assert_eq!(cpu.regs.sp, 0xBEED);
                assert_eq!(ctx.memory[0xBEEE], 0x79);
                assert_eq!(ctx.memory[0xBEED], 0x42);
            }
        }
    };
}

pub(crate) use test_rst;

macro_rules! test_invalid {
    ($byte:literal) => {
        test_decode!($byte, Opcode::INVALID($byte));
    };
}

pub(crate) use test_invalid;

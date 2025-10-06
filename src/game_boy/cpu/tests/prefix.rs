use crate::game_boy::cpu::{
    registers::Flags,
    tests::{END_INSTRUCTION, run_test},
};

macro_rules! prefix_mem {
    ($byte:expr, hl, $input:expr) => {
        &[0xCB, $byte, END_INSTRUCTION, $input]
    };
    ($byte:expr, $reg:ident, $input:expr) => {
        &[0xCB, $byte]
    };
}

macro_rules! prefix_cycles {
    (hl, true) => {
        4
    };
    (hl, false) => {
        3
    };
    ($reg:ident, $upcycle:literal) => {
        2
    };
}

macro_rules! get_reg {
    ($cpu:ident, $ctx:ident, hl) => {
        $ctx.memory[0x0003]
    };
    ($cpu:ident, $ctx:ident, $reg:ident) => {
        $cpu.regs.$reg
    };
}

macro_rules! test_prefix {
    ($byte:expr, $name:ident, $bit:literal, $upcycle:literal
     $reg:ident =>
     $($input:literal $($flag_init:ident)* -> $output:literal $($flag:ident)*)+) => {
        paste::paste! {
            $(
                #[test]
                fn [<test_0xcb_ $name _ $bit _ $reg _case_ $input  $($flag_init)*>]() {
                    let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, $input), |cpu| {
                        set_reg!(cpu, $reg, $input, 0x0003);
                        cpu.regs.f $(.[<set_ $flag_init>](true))*;
                    });
                    let expected_cycles = prefix_cycles!($reg, $upcycle);
                    assert_eq!(ctx.cycle_count, expected_cycles);
                    assert_eq!(get_reg!(cpu, ctx, $reg), $output);
                    let expected_flags = Flags::new() $(.[<with_ $flag>](true))*;
                    assert_eq!(cpu.regs.f, expected_flags);
                }
            )*
        }
    };
}

macro_rules! test_rlc {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, rlc, 0, true
            $reg =>
            0xff -> 0xff c
            0x00 -> 0x00 z
            0x80 -> 0x01 c
            0x79 -> 0xf2
        );
    };
}

macro_rules! test_rrc {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, rrc, 0, true
            $reg =>
            0xff -> 0xff c
            0x00 -> 0x00 z
            0x80 -> 0x40
            0x01 -> 0x80 c
            0x79 -> 0xbc c
        );
    };
}

macro_rules! test_rl {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, rl, 0, true
            $reg =>
            0xff -> 0xfe c
            0xff c -> 0xff c
            0x00 -> 0x00 z
            0x00 c -> 0x01
            0x80 -> 0x00 z c
            0x80 c -> 0x01 c
            0x79 -> 0xf2
            0x79 c -> 0xf3
        );
    };
}

macro_rules! test_rr {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, rr, 0, true
            $reg =>
            0xff -> 0x7f c
            0xff c -> 0xff c
            0x00 -> 0x00 z
            0x00 c -> 0x80
            0x80 -> 0x40
            0x80 c -> 0xc0
            0x01 -> 0x00 z c
            0x01 c -> 0x80 c
            0x79 -> 0x3c c
            0x79 c -> 0xbc c
        );
    };
}

macro_rules! test_sla {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, sla, 0, true
            $reg =>
            0xff -> 0xfe c
            0x00 -> 0x00 z
            0x80 -> 0x00 z c
            0x79 -> 0xf2
        );
    };
}

macro_rules! test_sra {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, sra, 0, true
            $reg =>
            0xff -> 0xff c
            0x00 -> 0x00 z
            0x80 -> 0xc0
            0x01 -> 0x00 z c
            0x79 -> 0x3c c
        );
    };
}

macro_rules! test_swap {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, swap, 0, true
            $reg =>
            0xff -> 0xff
            0x00 -> 0x00 z
            0x80 -> 0x08
            0x01 -> 0x10
            0x79 -> 0x97
        );
    };
}

macro_rules! test_srl {
    ($byte:expr, $reg:ident) => {
        test_prefix!($byte, srl, 0, true
            $reg =>
            0xff -> 0x7f c
            0x00 -> 0x00 z
            0x80 -> 0x40
            0x01 -> 0x00 z c
            0x79 -> 0x3c c
        );
    };
}

macro_rules! test_bit {
    ($byte:expr, $bit:literal, $reg:ident) => {
        test_prefix!($byte, bit, $bit, false
            $reg =>
            0xff -> 0xff h
            0x00 -> 0x00 z h
        );
        paste::paste! {
            #[test]
            fn [<test_0xcb_bit_ $bit _ $reg _case_0b01>]() {
                let input = 0b0101_0101;
                let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, input), |cpu| {
                    set_reg!(cpu, $reg, input, 0x0003);
                });
                assert_eq!(ctx.cycle_count, prefix_cycles!($reg, false));
                let zero = (input >> $bit) & 1 == 0;
                assert_eq!(cpu.regs.f, Flags::new().with_z(zero).with_h(true));
                assert_eq!(get_reg!(cpu, ctx, $reg), input);
            }
            #[test]
            fn [<test_0xcb_bit_ $bit _ $reg _case_0b10>]() {
                let input = 0b1010_1010;
                let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, input), |cpu| {
                    set_reg!(cpu, $reg, input, 0x0003);
                });
                assert_eq!(ctx.cycle_count, prefix_cycles!($reg, false));
                let zero = (input >> $bit) & 1 == 0;
                assert_eq!(cpu.regs.f, Flags::new().with_z(zero).with_h(true));
                assert_eq!(get_reg!(cpu, ctx, $reg), input);
            }
        }
    };
}

macro_rules! test_res {
    ($byte:expr, $bit:literal, $reg:ident) => {
        test_prefix!($byte, res, $bit, true
            $reg =>
            0x00 -> 0x00
        );
        paste::paste! {
            #[test]
            fn [<test_0xcb_res_ $bit _ $reg _case_0b01>]() {
                let input = 0b0101_0101;
                let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, input), |cpu| {
                    set_reg!(cpu, $reg, input, 0x0003);
                });
                assert_eq!(ctx.cycle_count, prefix_cycles!($reg, true));
                let bit = (get_reg!(cpu, ctx, $reg) >> $bit) & 1;
                assert_eq!(bit, 0);
                assert_eq!(cpu.regs.f, Flags::new());
            }
            #[test]
            fn [<test_0xcb_res_ $bit _ $reg _case_0b10>]() {
                let input = 0b1010_1010;
                let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, input), |cpu| {
                    set_reg!(cpu, $reg, input, 0x0003);
                });
                assert_eq!(ctx.cycle_count, prefix_cycles!($reg, true));
                let bit = (get_reg!(cpu, ctx, $reg) >> $bit) & 1;
                assert_eq!(bit, 0);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

macro_rules! test_set {
    ($byte:expr, $bit:literal, $reg:ident) => {
        test_prefix!($byte, set, $bit, true
            $reg =>
            0xff -> 0xff
        );
        paste::paste! {
            #[test]
            fn [<test_0xcb_set_ $bit _ $reg _case_0b01>]() {
                let input = 0b0101_0101;
                let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, input), |cpu| {
                    set_reg!(cpu, $reg, input, 0x0003);
                });
                assert_eq!(ctx.cycle_count, prefix_cycles!($reg, true));
                let bit = (get_reg!(cpu, ctx, $reg) >> $bit) & 1;
                assert_eq!(bit, 1);
                assert_eq!(cpu.regs.f, Flags::new());
            }
            #[test]
            fn [<test_0xcb_set_ $bit _ $reg _case_0b10>]() {
                let input = 0b1010_1010;
                let (cpu, ctx) = run_test(prefix_mem!($byte, $reg, input), |cpu| {
                    set_reg!(cpu, $reg, input, 0x0003);
                });
                assert_eq!(ctx.cycle_count, prefix_cycles!($reg, true));
                let bit = (get_reg!(cpu, ctx, $reg) >> $bit) & 1;
                assert_eq!(bit, 1);
                assert_eq!(cpu.regs.f, Flags::new());
            }
        }
    };
}

macro_rules! reg_bit {
    (b) => {
        0
    };
    (c) => {
        1
    };
    (d) => {
        2
    };
    (e) => {
        3
    };
    (h) => {
        4
    };
    (l) => {
        5
    };
    (hl) => {
        6
    };
    (a) => {
        7
    };
}

macro_rules! test_operand {
    ($reg:ident) => {
        test_rlc!((0 << 3) + reg_bit!($reg), $reg);
        test_rrc!((1 << 3) + reg_bit!($reg), $reg);
        test_rl!((2 << 3) + reg_bit!($reg), $reg);
        test_rr!((3 << 3) + reg_bit!($reg), $reg);
        test_sla!((4 << 3) + reg_bit!($reg), $reg);
        test_sra!((5 << 3) + reg_bit!($reg), $reg);
        test_swap!((6 << 3) + reg_bit!($reg), $reg);
        test_srl!((7 << 3) + reg_bit!($reg), $reg);
        test_index!(0b000, $reg);
        test_index!(0b001, $reg);
        test_index!(0b010, $reg);
        test_index!(0b011, $reg);
        test_index!(0b100, $reg);
        test_index!(0b101, $reg);
        test_index!(0b110, $reg);
        test_index!(0b111, $reg);
    };
}

macro_rules! test_index {
    ($bit:literal, $reg:ident) => {
        test_bit!((1 << 6) + ($bit << 3) + reg_bit!($reg), $bit, $reg);
        test_res!((2 << 6) + ($bit << 3) + reg_bit!($reg), $bit, $reg);
        test_set!((3 << 6) + ($bit << 3) + reg_bit!($reg), $bit, $reg);
    };
}

test_operand!(a);
test_operand!(b);
test_operand!(c);
test_operand!(d);
test_operand!(e);
test_operand!(h);
test_operand!(l);
test_operand!(hl);

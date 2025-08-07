use super::{
    opcode::{CBOpcode, Condition, Opcode, R8, R16mem},
    registers::{Reg8, Reg16},
};

const OPCODE_LOOKUP_TABLE: [Opcode; 256] = Opcode::generate_table();

impl Opcode {
    pub const fn lookup(data: u8) -> Self {
        OPCODE_LOOKUP_TABLE[data as usize]
    }

    const fn generate_table() -> [Self; 256] {
        let mut res = [Self::NOP; 256];
        let mut i = 0;
        while i < 256 {
            res[i] = Opcode::parse(i as u8);
            i += 1;
        }
        res
    }

    const fn parse(opcode: u8) -> Self {
        let attrs = OpcodeAttrs(opcode);
        match opcode & 0xC0 {
            // Block 0
            0x00 => match opcode & 0x0F {
                0x00 | 0x08 => match opcode >> 3 {
                    0x000 => Self::NOP,
                    0x001 => Self::LD_imm16_sp,
                    0x010 => Self::STOP,
                    0x011 => Self::JR_imm8,
                    _ => Self::JR_cond_imm8 { cond: attrs.cond() },
                },
                0x01 => Self::LD_r16_imm16 { dest: attrs.r16() },
                0x02 => Self::LD_r16mem_a {
                    r16mem: attrs.r16mem(),
                },
                0x03 => Self::INC_r16 { r16: attrs.r16() },
                0x04 | 0x0C => Self::INC_r8 { r8: attrs.r8l() },
                0x05 | 0x0D => Self::DEC_r8 { r8: attrs.r8l() },
                0x06 | 0x0E => Self::LD_r8_imm8 { r8: attrs.r8l() },
                0x07 | 0x0F => match opcode >> 3 {
                    0b000 => Self::RLCA,
                    0b001 => Self::RRCA,
                    0b010 => Self::RLA,
                    0b011 => Self::RRA,
                    0b100 => Self::DAA,
                    0b101 => Self::CPL,
                    0b110 => Self::SCF,
                    0b111 => Self::CCF,
                    _ => unreachable!(),
                },
                0x09 => Self::ADD_hl_r16 { r16: attrs.r16() },
                0x0A => Self::LD_a_r16mem {
                    r16mem: attrs.r16mem(),
                },
                0x0B => Self::DEC_r16 { r16: attrs.r16() },
                _ => unreachable!(),
            },
            // Block 1
            0x40 => match opcode {
                0b01110110 => Self::HALT,
                _ => Self::LD_r8_r8 {
                    dest: attrs.r8l(),
                    src: attrs.r8r(),
                },
            },
            // Block 2
            0x80 => match (opcode >> 3) & 0b111 {
                0b000 => Self::ADD_a_r8 { r8: attrs.r8r() },
                0b001 => Self::ADC_a_r8 { r8: attrs.r8r() },
                0b010 => Self::SUB_a_r8 { r8: attrs.r8r() },
                0b011 => Self::SBC_a_r8 { r8: attrs.r8r() },
                0b100 => Self::AND_a_r8 { r8: attrs.r8r() },
                0b101 => Self::XOR_a_r8 { r8: attrs.r8r() },
                0b110 => Self::OR_a_r8 { r8: attrs.r8r() },
                0b111 => Self::CP_a_r8 { r8: attrs.r8r() },
                _ => unreachable!(),
            },
            // Block 3
            0xC0 => match opcode & 0x0F {
                0x01 => Self::POP {
                    r16stk: attrs.r16stk(),
                },
                0x05 => Self::PUSH {
                    r16stk: attrs.r16stk(),
                },
                0x06 | 0x0E => match (opcode >> 3) & 0b111 {
                    0b000 => Self::ADD_a_imm8,
                    0b001 => Self::ADC_a_imm8,
                    0b010 => Self::SUB_a_imm8,
                    0b011 => Self::SBC_a_imm8,
                    0b100 => Self::AND_a_imm8,
                    0b101 => Self::XOR_a_imm8,
                    0b110 => Self::OR_a_imm8,
                    0b111 => Self::CP_a_imm8,
                    _ => unreachable!(),
                },
                0x07 | 0x0F => Self::RST {
                    tgt3: (opcode >> 3) & 0b111,
                },
                _ => match opcode & 0b111111 {
                    0b000000 | 0b001000 | 0b010000 | 0b011000 => {
                        Self::RET_cond { cond: attrs.cond() }
                    }
                    0b001001 => Self::RET,
                    0b011001 => Self::RETI,
                    0b000010 | 0b001010 | 0b010010 | 0b011010 => {
                        Self::JP_cond_imm16 { cond: attrs.cond() }
                    }
                    0b000011 => Self::JP_imm16,
                    0b101001 => Self::JP_hl,
                    0b000100 | 0b001100 | 0b010100 | 0b011100 => {
                        Self::CALL_cond_imm16 { cond: attrs.cond() }
                    }
                    0b001101 => Self::CALL_imm16,
                    0b001011 => Self::PREFIX,
                    0b100010 => Self::LDH_c_a,
                    0b100000 => Self::LDH_imm8_a,
                    0b101010 => Self::LD_imm16_a,
                    0b110010 => Self::LDH_a_c,
                    0b110000 => Self::LDH_a_imm8,
                    0b111010 => Self::LD_a_imm16,
                    0b101000 => Self::ADD_sp_imm8,
                    0b111000 => Self::LD_hl_spimm8,
                    0b111001 => Self::LD_sp_hl,
                    0b110011 => Self::DI,
                    0b111011 => Self::EI,
                    _ => Self::INVALID,
                },
            },
            _ => unreachable!(),
        }
    }
}
const CB_OPCODE_LOOKUP_TABLE: [CBOpcode; 256] = CBOpcode::generate_table();

impl CBOpcode {
    pub const fn lookup(data: u8) -> Self {
        CB_OPCODE_LOOKUP_TABLE[data as usize]
    }

    const fn generate_table() -> [Self; 256] {
        let mut res = [Self::RLC {
            r8: R8::Reg(Reg8::A),
        }; 256];
        let mut i = 0;
        while i < 256 {
            res[i] = CBOpcode::parse(i as u8);
            i += 1;
        }
        res
    }

    const fn parse(opcode: u8) -> Self {
        let attrs = OpcodeAttrs(opcode);
        let upper_bits = opcode >> 6;
        let r8 = attrs.r8r();
        let b3 = attrs.b3();
        match (upper_bits, b3) {
            (0, 0) => Self::RLC { r8 },
            (0, 1) => Self::RRC { r8 },
            (0, 2) => Self::RL { r8 },
            (0, 3) => Self::RR { r8 },
            (0, 4) => Self::SLA { r8 },
            (0, 5) => Self::SRA { r8 },
            (0, 6) => Self::SWAP { r8 },
            (0, 7) => Self::SRL { r8 },
            (1, b3) => Self::BIT { b3, r8 },
            (2, b3) => Self::RES { b3, r8 },
            (3, b3) => Self::SET { b3, r8 },
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct OpcodeAttrs(u8);

impl OpcodeAttrs {
    const fn r16(self) -> Reg16 {
        use Reg16::*;
        match (self.0 >> 4) & 0b11 {
            0b00 => BC,
            0b01 => DE,
            0b10 => HL,
            0b11 => SP,
            _ => unreachable!(),
        }
    }
    const fn r16mem(self) -> R16mem {
        use R16mem::*;
        match (self.0 >> 4) & 0b11 {
            0b00 => BC,
            0b01 => DE,
            0b10 => HLi,
            0b11 => HLd,
            _ => unreachable!(),
        }
    }
    const fn r16stk(self) -> Reg16 {
        use Reg16::*;
        match (self.0 >> 4) & 0b11 {
            0b00 => BC,
            0b01 => DE,
            0b10 => HL,
            0b11 => AF,
            _ => unreachable!(),
        }
    }
    const fn r8(self, shift: u8) -> R8 {
        use R8::*;
        use Reg8::*;
        match (self.0 >> shift) & 0b111 {
            0b000 => Reg(A),
            0b001 => Reg(B),
            0b010 => Reg(C),
            0b011 => Reg(D),
            0b100 => Reg(E),
            0b101 => Reg(H),
            0b110 => Reg(L),
            0b111 => HLaddr,
            _ => unreachable!(),
        }
    }
    const fn r8l(self) -> R8 {
        self.r8(3)
    }
    const fn r8r(self) -> R8 {
        self.r8(0)
    }
    const fn cond(self) -> Condition {
        use Condition::*;
        match (self.0 >> 3) & 0b11 {
            0b00 => NZ,
            0b01 => Z,
            0b10 => NC,
            0b11 => C,
            _ => unreachable!(),
        }
    }
    const fn b3(self) -> u8 {
        (self.0 >> 3) & 0b111
    }
}

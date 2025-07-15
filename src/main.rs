use core::cpu::opcode::Opcode;

mod core;

fn main() {
    for i in 0..=u8::MAX {
        println!("{i:02X} -> {}", Opcode::lookup(i))
    }
}

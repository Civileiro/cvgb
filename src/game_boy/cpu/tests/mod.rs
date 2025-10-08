mod decode_recode;
mod instructions;

#[macro_use]
mod macros;
mod prefix;

use crate::game_boy::{
    Events,
    context::interrupts::{Interrupt, InterruptFlags},
    events::Event,
};

use super::{CPUState, Cpu, CpuContext, opcode::Opcode};

#[derive(Debug)]
pub struct StubContext {
    pub cycle_count: usize,
    pub memory: Box<[u8]>,
    pub events: Events,
}

impl StubContext {
    fn with_memory(mut memory: Vec<u8>) -> Self {
        memory.resize(0x1_0000, 0);
        Self {
            cycle_count: 0,
            memory: memory.into_boxed_slice(),
            events: Events::default(),
        }
    }
}

const END_INSTRUCTION: u8 = 0xFC;
const END_OPCODE: Opcode = Opcode::lookup(END_INSTRUCTION);

pub fn run_test(init_memory: &[u8], init: impl FnOnce(&mut Cpu)) -> (Cpu, StubContext) {
    let mut memory = init_memory.to_vec();
    memory.push(END_INSTRUCTION);
    let mut ctx = StubContext::with_memory(memory);

    let mut cpu = Cpu::default();
    cpu.step(&mut ctx);
    ctx.cycle_count = 0;
    init(&mut cpu);

    while cpu.opcode != END_OPCODE && cpu.state.is_normal() {
        cpu.step(&mut ctx);
    }
    (cpu, ctx)
}

impl CpuContext for StubContext {
    fn cycle_read_itrs(&mut self, addr: u16) -> (u8, InterruptFlags) {
        self.cycle();
        let data = self.memory[addr as usize];
        (data, InterruptFlags::new())
    }

    fn cycle_write_itrs(&mut self, addr: u16, data: u8) -> InterruptFlags {
        self.cycle();
        self.memory[addr as usize] = data;
        InterruptFlags::new()
    }

    fn cycle_state_itrs(&mut self, _state: CPUState) -> InterruptFlags {
        self.cycle_count += 1;
        InterruptFlags::new()
    }

    fn ack_interrupt(&mut self, _: Interrupt) {}

    fn speed_switch(&mut self) {}

    fn has_pressed_input(&self) -> bool {
        false
    }
    fn has_speed_switch_armed(&self) -> bool {
        false
    }
    fn reset_div(&mut self) {}

    fn signal_event(&mut self, event: Event) {
        self.events.signal_event(event);
    }
}

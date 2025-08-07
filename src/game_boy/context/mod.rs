use interrupts::{Interrupt, InterruptFlags};
use p1::P1;

use super::{
    cartridge::Cartridge, cpu::CpuContext, events::Events, input::Input, time::SystemTime,
};

pub mod interrupts;
mod p1;

#[derive(Debug)]
pub struct Context {
    time: SystemTime,
    events: Events,
    cartridge: Cartridge,
    boot_rom_enabled: bool,
    p1: P1,
    interrupts: InterruptFlags,
    interrupt_enable: InterruptFlags,
}

impl CpuContext for Context {
    fn cycle_read_itrs(&mut self, addr: u16) -> (u8, InterruptFlags) {
        todo!()
    }

    fn cycle_write_itrs(&mut self, addr: u16, data: u8) -> InterruptFlags {
        todo!()
    }

    fn cycle_state_itrs(&mut self, state: super::cpu::CPUState) -> InterruptFlags {
        todo!()
    }

    fn ack_interrupt(&mut self, itr: Interrupt) {
        match itr {
            Interrupt::VBLANK => self.interrupts.set_vblank(false),
            Interrupt::LCD => self.interrupts.set_lcd(false),
            Interrupt::TIMER => self.interrupts.set_timer(false),
            Interrupt::SERIAL => self.interrupts.set_serial(false),
            Interrupt::JOYPAD => self.interrupts.set_joypad(false),
        }
    }

    fn has_interrupt(&mut self) -> bool {
        self.interrupts.has_interrupt()
    }

    fn speed_switch(&mut self) {
        todo!()
    }

    fn has_pressed_input(&self) -> bool {
        todo!()
    }
}

impl Context {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            time: Default::default(),
            events: Default::default(),
            cartridge,
            boot_rom_enabled: true,
            p1: Default::default(),
            interrupts: Default::default(),
            interrupt_enable: Default::default(),
        }
    }
    pub fn set_input(&mut self, input: Input) {
        if self.p1.set_input(input) {
            self.interrupts.set_joypad(true);
        }
    }
    pub fn fetch_clear_events(&mut self) -> Events {
        let res = self.events;
        self.events = Events::new();
        res
    }
    pub fn system_time(&self) -> SystemTime {
        self.time
    }
    pub fn press_key(&mut self, input: Input) {
        if self.p1.press(input) {
            self.interrupts.set_joypad(true);
        }
    }
    pub fn unpress_key(&mut self, input: Input) {
        if self.p1.unpress(input) {
            self.interrupts.set_joypad(true);
        }
    }
}

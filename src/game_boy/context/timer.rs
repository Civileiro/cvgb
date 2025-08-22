use modular_bitfield::prelude::*;

use super::interrupts::InterruptFlags;

#[derive(Debug, Default)]
pub struct Timer {
    sys_clock: u16,
    /// Timer Counter
    tima: u8,
    // Timer Modulo
    tma: u8,
    // Timer Control
    tac: Tac,
    overflowed: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
struct Tac {
    pub clock_select: B2,
    pub enable: bool,
    #[skip]
    __: B5,
}

impl Timer {
    pub fn div(&self) -> u8 {
        (self.sys_clock >> 6) as u8
    }
    pub fn frequency_mask(&self) -> u16 {
        match self.tac.clock_select() {
            0b00 => 0b10000000,
            0b01 => 0b00000010,
            0b10 => 0b00001000,
            0b11 => 0b00100000,
            _ => unreachable!(),
        }
    }
    fn tick(&mut self) {
        let (tima, overflow) = self.tima.overflowing_add(1);
        self.tima = tima;
        self.overflowed = overflow;
    }
    fn clock(&mut self, itrs: &mut InterruptFlags) {
        if self.overflowed {
            self.overflowed = false;
            self.tima = self.tma;
            itrs.set_timer(true);
        }
        self.sys_clock = self.sys_clock.wrapping_add(1);
    }
    fn selected_bit(&self) -> bool {
        self.sys_clock & self.frequency_mask() != 0
    }
    fn watch_selected_bit<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let prev_bit = self.selected_bit();
        let res = f(self);
        let new_bit = self.selected_bit();
        if self.tac.enable() && prev_bit && !new_bit {
            self.tick();
        }
        res
    }

    pub fn cycle(&mut self, itrs: &mut InterruptFlags) {
        self.watch_selected_bit(|slf| slf.clock(itrs))
    }
    pub fn cycle_read_div(&mut self, itrs: &mut InterruptFlags) -> u8 {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            slf.div()
        })
    }
    pub fn cycle_write_div(&mut self, itrs: &mut InterruptFlags, _data: u8) {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            slf.sys_clock = 0;
        })
    }
    pub fn cycle_read_tima(&mut self, itrs: &mut InterruptFlags) -> u8 {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            slf.tima
        })
    }
    pub fn cycle_write_tima(&mut self, itrs: &mut InterruptFlags, data: u8) {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            // Writing to TIMA during an overflow cycle causes it to be ignored
            slf.overflowed = false;
            slf.tima = data
        })
    }
    pub fn cycle_read_tma(&mut self, itrs: &mut InterruptFlags) -> u8 {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            slf.tma
        })
    }
    pub fn cycle_write_tma(&mut self, itrs: &mut InterruptFlags, data: u8) {
        self.watch_selected_bit(|slf| {
            slf.tma = data;
            slf.clock(itrs);
        })
    }
    pub fn cycle_read_tac(&mut self, itrs: &mut InterruptFlags) -> u8 {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            slf.tac.into()
        })
    }
    pub fn cycle_write_tac(&mut self, itrs: &mut InterruptFlags, data: u8) {
        self.watch_selected_bit(|slf| {
            slf.clock(itrs);
            slf.tac = data.into();
        })
    }
}

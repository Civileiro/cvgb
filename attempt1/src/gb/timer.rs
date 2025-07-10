use std::{cell::RefCell, rc::Rc};

use modular_bitfield::prelude::*;

use super::{
    Addressable, bus::AddressingError, constants::INTERRUPT_TIMER_BIT, core::CPUIORegisters,
    variant::GameBoyVariant,
};

#[derive(Debug, Default)]
struct TimerData {
    sys_clock: u16,
    tima: u8,
    tma: u8,
    tac: u8,
    overflowed: bool,
}

#[bitfield(bits = 8)]
struct Tac {
    pub clock_select: B2,
    pub enable: bool,
    #[skip]
    __: B5,
}

impl TimerData {
    /// Advances the timer 1 M-Cycle
    fn clock(&mut self) -> bool {
        let mut res = false;
        if self.overflowed {
            self.overflowed = false;
            self.tima = self.tma;
            res = true;
        }
        let prev_clock = self.sys_clock;
        self.sys_clock = self.sys_clock.wrapping_add(1);
        let tac_flags = self.tac_flags();
        if !tac_flags.enable() {
            return res;
        }
        let freq_mask = Self::freq_mask(tac_flags);
        if (prev_clock ^ self.sys_clock) & freq_mask != 0 {
            self.tick()
        }
        res
    }
    fn div(&self) -> u8 {
        (self.sys_clock >> 6) as u8
    }
    fn tac_flags(&self) -> Tac {
        Tac::from_bytes([self.tac])
    }
    fn freq_mask(tac: Tac) -> u16 {
        match tac.clock_select() {
            0b00 => 0b10000000,
            0b01 => 0b00100000,
            0b10 => 0b00001000,
            0b11 => 0b00000010,
            _ => unreachable!(),
        }
    }
    fn selected_bit(&self) -> bool {
        let tac_flags = self.tac_flags();
        let mask = Self::freq_mask(tac_flags);
        (self.sys_clock & mask) != 0
    }
    fn enabled(&self) -> bool {
        self.tac & 0b100 != 0
    }
    fn try_tick(&mut self) {
        if self.tac_flags().enable() {
            self.tick();
        }
    }
    fn tick(&mut self) {
        let (tima, overflow) = self.tima.overflowing_add(1);
        self.tima = tima;
        self.overflowed = overflow;
    }
}

pub struct Timer {
    variant: GameBoyVariant,
    data: RefCell<TimerData>,
    cpu_io: Rc<CPUIORegisters>,
}

impl Timer {
    pub fn new(variant: GameBoyVariant, cpu_io: Rc<CPUIORegisters>) -> Self {
        Self {
            variant,
            data: RefCell::new(TimerData::default()),
            cpu_io,
        }
    }
    /// Runs on every machine clock, unless CPU is in STOP mode
    pub fn clock(&self) {
        if self.data.borrow_mut().clock() {
            self.cpu_io
                .set_interrupts(self.cpu_io.get_interrupts() | INTERRUPT_TIMER_BIT);
        }
    }
}

impl Addressable for Timer {
    fn size(&self) -> usize {
        4
    }

    fn const_read(&self, addr: u16) -> Result<u8, super::bus::AddressingError> {
        let timer = self.data.borrow();
        match addr {
            0x00 => Ok(timer.div()),
            0x01 => Ok(timer.tima),
            0x02 => Ok(timer.tma),
            0x03 => Ok(timer.tac),
            _ => Err(AddressingError::Unmapped),
        }
    }

    fn write(&self, addr: u16, data: u8) -> Result<(), super::bus::AddressingError> {
        let mut timer = self.data.borrow_mut();
        match addr {
            0x00 => {
                let old_bit = timer.selected_bit();
                // "Writing any value to this register resets it to $00"
                timer.sys_clock = 0x00;
                let new_bit = timer.selected_bit();
                if old_bit != new_bit {
                    timer.try_tick();
                }
                Ok(())
            }
            0x01 => {
                timer.tima = data;
                Ok(())
            }
            0x02 => {
                timer.tma = data;
                Ok(())
            }
            0x03 => {
                let old_bit = timer.selected_bit();
                let old_enable = timer.enabled();
                timer.tac = data;
                let new_bit = timer.selected_bit();
                let new_enable = timer.enabled();
                if old_bit != new_bit {
                    // changing bit causes a tick if timer is enabled
                    timer.try_tick();
                } else if !self.variant.is_color_variant() {
                    // monochrome consoles will tick when (bit && enable) goes low
                    if (old_bit && old_enable) && !(new_bit && new_enable) {
                        timer.tick();
                    }
                }
                Ok(())
            }
            _ => Err(AddressingError::Unmapped),
        }
    }
}

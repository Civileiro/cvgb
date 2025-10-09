use enum_assoc::Assoc;
use modular_bitfield::prelude::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Assoc)]
#[func(pub fn handler_address(&self) -> u16)]
pub enum Interrupt {
    #[assoc(handler_address = 0x40)]
    VBLANK,
    #[assoc(handler_address = 0x48)]
    LCD,
    #[assoc(handler_address = 0x50)]
    TIMER,
    #[assoc(handler_address = 0x58)]
    SERIAL,
    #[assoc(handler_address = 0x60)]
    JOYPAD,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InterruptFlags {
    pub vblank: bool,
    pub lcd: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
    #[skip]
    __: B3,
}

impl InterruptFlags {
    pub fn highest_priority(&self) -> Option<Interrupt> {
        if self.vblank() {
            Some(Interrupt::VBLANK)
        } else if self.lcd() {
            Some(Interrupt::LCD)
        } else if self.timer() {
            Some(Interrupt::TIMER)
        } else if self.serial() {
            Some(Interrupt::SERIAL)
        } else if self.joypad() {
            Some(Interrupt::JOYPAD)
        } else {
            None
        }
    }
    pub fn has_interrupt(&self) -> bool {
        self.highest_priority().is_some()
    }
    pub fn clear(&mut self) {
        *self = Self::new()
    }
}

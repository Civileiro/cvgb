use std::cell::Cell;

use super::{Addressable, GameBoyVariant, bus::AddressingError, memory::Memory};

pub struct BootRom {
    rom: &'static [u8],
    enabled: Cell<bool>,
}

impl BootRom {
    pub fn new(gb_variant: GameBoyVariant) -> Self {
        let rom_bytes = match gb_variant {
            GameBoyVariant::DMG0 => include_bytes!("dmg0.bin"),
            GameBoyVariant::DMG => include_bytes!("dmg.bin"),
            _ => todo!("include all variant bios'"),
        };
        Self {
            rom: rom_bytes,
            enabled: Cell::new(true),
        }
    }
    pub fn disable(&self) {
        self.enabled.set(false)
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }
}

impl Addressable for BootRom {
    fn size(&self) -> usize {
        self.rom.len()
    }

    fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
        // The range $0100-$01FF is for the catridge header
        if (addr >> 8) == 1 {
            Err(AddressingError::Unmapped)
        } else if !self.is_enabled() {
            Err(AddressingError::Disabled)
        } else {
            Ok(self.rom[addr as usize])
        }
    }

    fn write(&self, _addr: u16, _data: u8) -> Result<(), AddressingError> {
        Err(AddressingError::Unmapped)
    }
}

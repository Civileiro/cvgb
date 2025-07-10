use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{Addressable, bus::AddressingError, ppu::VRam, variant::GameBoyVariant};

pub struct WRam {
    bank: Cell<u8>,
    memory: RefCell<Box<[u8]>>,
}

impl WRam {
    pub fn new(variant: GameBoyVariant) -> Self {
        let size = if variant.is_color_variant() {
            32 << 10
        } else {
            8 << 10
        };
        let memory = RefCell::new(vec![0u8; size].into_boxed_slice());
        Self {
            bank: Cell::new(0),
            memory,
        }
    }
    fn actual_bank(&self) -> u16 {
        let bank = self.bank.get();
        if bank == 0 { 1 } else { bank.into() }
    }
}

impl Addressable for WRam {
    fn size(&self) -> usize {
        0x2000
    }

    fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
        match addr {
            0x0000..=0x0FFF => Ok(self.memory.borrow()[addr as usize]),
            0x1000..=0x1FFF => {
                let addr = addr + (0x1000 * (self.actual_bank() - 1));
                Ok(self.memory.borrow()[addr as usize])
            }
            _ => Err(AddressingError::Unmapped),
        }
    }

    fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError> {
        match addr {
            0x0000..=0x0FFF => {
                self.memory.borrow_mut()[addr as usize] = data;
                Ok(())
            }
            0x1000..=0x1FFF => {
                let addr = addr + (0x1000 * (self.actual_bank() - 1));
                self.memory.borrow_mut()[addr as usize] = data;
                Ok(())
            }
            _ => Err(AddressingError::Unmapped),
        }
    }
}

pub struct WRamBank {
    variant: Rc<GameBoyVariant>,
    wram: Rc<WRam>,
}

impl WRamBank {
    pub fn new(variant: Rc<GameBoyVariant>, wram: Rc<WRam>) -> Self {
        Self { variant, wram }
    }
}

impl Addressable for WRamBank {
    fn size(&self) -> usize {
        1
    }

    fn const_read(&self, _addr: u16) -> Result<u8, AddressingError> {
        if self.variant.is_dmg_compatible() {
            Err(AddressingError::Unmapped)
        } else {
            let bank = self.wram.bank.get();
            Ok(0b11111000 | bank)
        }
    }

    fn write(&self, _addr: u16, data: u8) -> Result<(), AddressingError> {
        if self.variant.is_dmg_compatible() {
            Err(AddressingError::Unmapped)
        } else {
            let bank = data & 0b111;
            self.wram.bank.set(bank);
            Ok(())
        }
    }
}

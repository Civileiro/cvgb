use std::{cell::RefCell, rc::Rc};

use super::{Addressable, Bus, bus::AddressingError, variant::GameBoyVariant};

struct DMAData {
    variant: GameBoyVariant,
    fetched_data: Option<u8>,
    curr: u16,
    start: u16,
    end: u16,
    bus: Rc<Bus>,
}

impl DMAData {
    fn new(variant: GameBoyVariant, bus: Rc<Bus>) -> Self {
        Self {
            variant,
            fetched_data: None,
            curr: 0,
            start: 0,
            end: 0,
            bus,
        }
    }
    fn set(&mut self, data: u8) {
        let start = (data as u16) * 0x100;
        self.curr = start;
        self.end = start + 0x100;
    }
    fn read_addr(&self) -> u16 {
        self.start + self.curr
    }
    fn write_addr(&self) -> u16 {
        0xFE00 + self.curr
    }
    fn active(&self) -> bool {
        self.read_addr() < self.end || self.fetched_data.is_some()
    }
    fn clock(&mut self) {
        if !self.active() {
            return;
        }
        if let Some(data) = self.fetched_data.take() {
            self.bus.write(self.write_addr(), data);
        }
        if self.read_addr() != self.end {
            self.fetched_data = Some(self.bus.read(self.read_addr()));
            self.curr += 1;
        }
    }
}

pub struct Dma {
    data: RefCell<DMAData>,
}

impl Dma {
    pub fn new(variant: GameBoyVariant, bus: Rc<Bus>) -> Self {
        Self {
            data: RefCell::new(DMAData::new(variant, bus)),
        }
    }
    pub fn clock(&self) {
        self.data.borrow_mut().clock()
    }
}

impl Addressable for Dma {
    fn size(&self) -> usize {
        1
    }

    fn const_read(&self, _addr: u16) -> Result<u8, AddressingError> {
        Err(AddressingError::Unmapped)
    }

    fn write(&self, _addr: u16, data: u8) -> Result<(), AddressingError> {
        self.data.borrow_mut().set(data);
        Ok(())
    }
}

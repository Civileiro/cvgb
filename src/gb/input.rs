use std::cell::RefCell;

use super::{Addressable, bus::AddressingError};

struct JoypadInputData {
    dpad_nibble: u8,
    buttons_nibble: u8,
    select_nibble: u8,
}

impl JoypadInputData {
    fn new() -> Self {
        Self {
            dpad_nibble: 0x0F,
            buttons_nibble: 0x0F,
            select_nibble: 0xF0,
        }
    }
    fn select_buttons(&self) -> bool {
        self.select_nibble & 0b0010_0000 == 0
    }
    fn select_dpad(&self) -> bool {
        self.select_nibble & 0b0001_0000 == 0
    }

    fn p1(&self) -> u8 {
        let mut lower_nibble = 0x0F;
        if self.select_buttons() {
            lower_nibble &= self.buttons_nibble;
        }
        if self.select_dpad() {
            lower_nibble &= self.dpad_nibble;
        }
        self.select_nibble | lower_nibble
    }

    fn set(&mut self, data: u8) {
        self.select_nibble = 0xC0 | (data & 0xF0);
    }
}

pub struct JoypadInput {
    data: RefCell<JoypadInputData>,
}

impl JoypadInput {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(JoypadInputData::new()),
        }
    }
    pub fn set_down(&self) {
        self.data.borrow_mut().dpad_nibble &= 0b0111
    }
    pub fn set_up(&self) {
        self.data.borrow_mut().dpad_nibble &= 0b1011
    }
    pub fn set_left(&self) {
        self.data.borrow_mut().dpad_nibble &= 0b1101
    }
    pub fn set_right(&self) {
        self.data.borrow_mut().dpad_nibble &= 0b1110
    }
    pub fn reset_down(&self) {
        self.data.borrow_mut().dpad_nibble |= 0b1000
    }
    pub fn reset_up(&self) {
        self.data.borrow_mut().dpad_nibble |= 0b0100
    }
    pub fn reset_left(&self) {
        self.data.borrow_mut().dpad_nibble |= 0b0010
    }
    pub fn reset_right(&self) {
        self.data.borrow_mut().dpad_nibble |= 0b0001
    }
    pub fn set_start(&self) {
        self.data.borrow_mut().buttons_nibble &= 0b0111
    }
    pub fn set_select(&self) {
        self.data.borrow_mut().buttons_nibble &= 0b1011
    }
    pub fn set_b(&self) {
        self.data.borrow_mut().buttons_nibble &= 0b1101
    }
    pub fn set_a(&self) {
        self.data.borrow_mut().buttons_nibble &= 0b1110
    }
    pub fn reset_start(&self) {
        self.data.borrow_mut().buttons_nibble |= 0b1000
    }
    pub fn reset_select(&self) {
        self.data.borrow_mut().buttons_nibble |= 0b0100
    }
    pub fn reset_b(&self) {
        self.data.borrow_mut().buttons_nibble |= 0b0010
    }
    pub fn reset_a(&self) {
        self.data.borrow_mut().buttons_nibble |= 0b0001
    }
}

impl Addressable for JoypadInput {
    fn size(&self) -> usize {
        1
    }

    fn const_read(&self, _addr: u16) -> Result<u8, AddressingError> {
        Ok(self.data.borrow().p1())
    }

    fn write(&self, _addr: u16, data: u8) -> Result<(), AddressingError> {
        self.data.borrow_mut().set(data);
        Ok(())
    }
}

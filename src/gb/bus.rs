use std::cell::RefCell;

pub struct Bus {
    ram: RefCell<[u8; 64 << 10]>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: [0; 64 << 10].into(),
        }
    }
    pub fn read(&self, addr: u16) -> u8 {
        let r = self.ram.borrow();
        r[addr as usize]
    }
    pub fn write(&self, addr: u16, data: u8) {
        let mut r = self.ram.borrow_mut();
        r[addr as usize] = data;
    }
}

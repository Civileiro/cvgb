use std::{
    cell::RefCell,
    cmp::Reverse,
    ops::RangeInclusive,
    rc::{Rc, Weak},
};

use thiserror::Error;

#[derive(Debug)]
struct BusComponent {
    pub range: RangeInclusive<u16>,
    pub priority: i32,
    pub memory: Weak<dyn Addressable>,
}

#[derive(Debug)]
pub struct Bus {
    components: RefCell<Vec<BusComponent>>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            components: RefCell::new(Vec::new()),
        }
    }
    pub fn plug_memory_with_priority(
        &self,
        memory: Weak<dyn Addressable>,
        range: RangeInclusive<u16>,
        priority: i32,
    ) -> bool {
        let mut components = self.components.borrow_mut();
        components.push(BusComponent {
            range,
            priority,
            memory,
        });
        // Highest priority go first
        components.sort_unstable_by_key(|comp| Reverse(comp.priority));
        true
    }
    pub fn plug_memory(&self, memory: Weak<dyn Addressable>, range: RangeInclusive<u16>) -> bool {
        self.plug_memory_with_priority(memory, range, 0)
    }
    /// Read from an address on the bus
    pub fn read(&self, addr: u16) -> u8 {
        let mut components = self.components.borrow_mut();

        let mut i = 0;
        while i < components.len() {
            let comp = &components[i];
            if !comp.range.contains(&addr) {
                i += 1;
                continue;
            }
            let Some(memory) = comp.memory.upgrade() else {
                components.remove(i);
                continue;
            };
            match memory.read(addr - comp.range.start()) {
                Ok(val) => return val,
                Err(AddressingError::Busy) => return 0xFF,
                Err(AddressingError::Unmapped | AddressingError::Disabled) => i += 1,
                Err(AddressingError::Unplugged) => {
                    components.remove(i);
                    continue;
                }
            }
        }
        // No component connected to this address
        0xFF
    }
    /// Read from an address on the bus, without side effects
    pub fn const_read(&self, addr: u16) -> u8 {
        let components = self.components.borrow();

        for comp in components.iter() {
            if comp.range.contains(&addr) {
                let Some(memory) = comp.memory.upgrade() else {
                    continue;
                };
                match memory.const_read(addr - comp.range.start()) {
                    Ok(val) => return val,
                    Err(AddressingError::Busy) => return 0xFF,
                    Err(_) => continue,
                }
            }
        }
        // No component connected to this address
        0xFF
    }
    /// Write to an address on the bus
    pub fn write(&self, addr: u16, data: u8) {
        let mut components = self.components.borrow_mut();

        let mut i = 0;
        while i < components.len() {
            let comp = &components[i];
            if !comp.range.contains(&addr) {
                i += 1;
                continue;
            }
            let Some(memory) = comp.memory.upgrade() else {
                components.remove(i);
                continue;
            };
            match memory.write(addr - comp.range.start(), data) {
                Ok(()) => break,
                Err(AddressingError::Busy) => break,
                Err(AddressingError::Unmapped | AddressingError::Disabled) => i += 1,
                Err(AddressingError::Unplugged) => {
                    components.remove(i);
                    continue;
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum AddressingError {
    #[error("tried accessing unmapped memory")]
    Unmapped,
    #[error("another component is using this memory")]
    Busy,
    #[error("component is not accessible anymore")]
    Unplugged,
    #[error("this address is currently disabled")]
    Disabled,
}

pub trait Addressable {
    /// Size of the Addressable object. May still return Unmapped when in range.
    /// Out of bounds access may return Unmapped or panic!
    fn size(&self) -> usize;
    /// Try to read at address
    fn read(&self, addr: u16) -> Result<u8, AddressingError> {
        self.const_read(addr)
    }
    /// Try to read at address without causing side effects
    fn const_read(&self, addr: u16) -> Result<u8, AddressingError>;
    /// Try to write to address
    fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError>;
}

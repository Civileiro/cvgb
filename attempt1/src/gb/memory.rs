use std::{
    cell::{Cell, RefCell},
    fmt::Debug,
};

use super::{Addressable, bus::AddressingError};

pub struct Memory<T> {
    size: usize,
    inner: RefCell<T>,
}

impl<T: Debug> Debug for Memory<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memory")
            .field("size", &self.size)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> Memory<T> {
    pub fn new(inner: T, size: usize) -> Self {
        Self {
            size,
            inner: RefCell::new(inner),
        }
    }
    pub fn size(&self) -> usize {
        self.size
    }
}

macro_rules! impl_default_addressable {
    () => {
        fn size(&self) -> usize {
            self.size
        }
        fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
            if let Ok(arr) = self.inner.try_borrow() {
                Ok(arr[addr as usize])
            } else {
                Err(AddressingError::Busy)
            }
        }

        fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError> {
            if let Ok(mut arr) = self.inner.try_borrow_mut() {
                arr[addr as usize] = data;
                Ok(())
            } else {
                Err(AddressingError::Busy)
            }
        }
    };
}
impl Addressable for Memory<Vec<u8>> {
    impl_default_addressable!();
}
impl<const N: usize> Addressable for Memory<[u8; N]> {
    impl_default_addressable!();
}
impl Addressable for Memory<Box<[u8]>> {
    impl_default_addressable!();
}

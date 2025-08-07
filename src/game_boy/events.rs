use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Events {
    vblank: bool,
    breakpoint: bool,
    #[skip]
    __: B6,
}

impl Events {
    pub fn is_empty(self) -> bool {
        let byte: u8 = self.into();
        byte == 0
    }
}

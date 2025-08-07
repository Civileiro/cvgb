use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Input: u8 {
        const RIGHT = 0b0000_0001;
        const LEFT = 0b0000_0010;
        const UP = 0b0000_0100;
        const DOWN = 0b0000_1000;
        const A = 0b0001_0000;
        const B = 0b0010_0000;
        const SELECT = 0b0100_0000;
        const START = 0b1000_0000;
    }
}

impl Input {
    /// The nibble representing the buttons
    /// An unset bit means that button is pressed
    pub fn buttons_nibble(self) -> u8 {
        let byte: u8 = self.bits();
        !((byte >> 4) & 0b1111)
    }
    /// The nibble representing the dpad
    /// An unset bit means that direction is pressed
    pub fn dpad_nibble(self) -> u8 {
        let byte: u8 = self.bits();
        !(byte & 0b1111)
    }
}

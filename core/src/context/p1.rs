const SELECT_BUTTONS_MASK: u8 = 0b0010_0000;
const SELECT_DPAD_MASK: u8 = 0b0001_0000;

#[derive(Debug, Default)]
/// The Game Boy P1 register contains all the input information
pub struct P1 {
    input: P1Reg,
    select_buttons: bool,
    select_dpad: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct P1Reg(u8);

impl P1Reg {
    pub fn bits(self) -> u8 {
        self.0
    }
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
    pub fn press(&mut self, input: Input) {
        self.0 |= input as u8
    }
    pub fn unpress(&mut self, input: Input) {
        self.0 &= !(input as u8)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Input {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Up = 0b0000_0100,
    Down = 0b0000_1000,
    A = 0b0001_0000,
    B = 0b0010_0000,
    Select = 0b0100_0000,
    Start = 0b1000_0000,
}

impl P1 {
    pub fn read(&self) -> u8 {
        let mut res = 0xFF;
        if self.select_buttons {
            res &= !SELECT_BUTTONS_MASK;
            res &= self.input.buttons_nibble()
        }
        if self.select_dpad {
            res &= !SELECT_DPAD_MASK;
            res &= self.input.dpad_nibble()
        }
        res
    }
    pub fn write(&mut self, val: u8) {
        self.select_buttons = val & SELECT_BUTTONS_MASK == 0;
        self.select_dpad = val & SELECT_DPAD_MASK == 0;
    }
    /// When the interrupts line goes from true to false, the joypad interrupt should be triggered
    fn interrupt_line(&self) -> bool {
        self.read() & 0x0F == 0x0F
    }
    pub fn has_pressed_input(&self) -> bool {
        !self.interrupt_line()
    }

    fn watch_interrupt_line(&mut self, f: impl FnOnce(&mut Self)) -> bool {
        let old_line = self.interrupt_line();
        f(self);
        let new_line = self.interrupt_line();
        old_line && !new_line
    }

    /// Presses button(s), returns true if the joypad interrupt was triggered
    pub fn press(&mut self, input: Input) -> bool {
        self.watch_interrupt_line(|slf| slf.input.press(input))
    }

    /// Unpresses button(s), returns true if the joypad interrupt was triggered
    pub fn unpress(&mut self, input: Input) -> bool {
        self.watch_interrupt_line(|slf| slf.input.unpress(input))
    }
}

use crate::game_boy::input::Input;

const SELECT_BUTTONS_MASK: u8 = 0b0010_0000;
const SELECT_DPAD_MASK: u8 = 0b0001_0000;

#[derive(Debug, Default)]
/// The Game Boy P1 register contains all the input information
pub struct P1 {
    input: Input,
    select_buttons: bool,
    select_dpad: bool,
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

    fn watch_interrupt_line(&mut self, f: impl FnOnce(&mut Self)) -> bool {
        let old_line = self.interrupt_line();
        f(self);
        let new_line = self.interrupt_line();
        old_line && !new_line
    }

    /// Sets the input information, returns true if the joypad interrupt was triggered
    pub fn set_input(&mut self, input: Input) -> bool {
        self.watch_interrupt_line(|slf| slf.input = input)
    }

    /// Presses button(s), returns true if the joypad interrupt was triggered
    pub fn press(&mut self, input: Input) -> bool {
        self.watch_interrupt_line(|slf| slf.input |= input)
    }

    /// Unpresses button(s), returns true if the joypad interrupt was triggered
    pub fn unpress(&mut self, input: Input) -> bool {
        self.watch_interrupt_line(|slf| slf.input &= !input)
    }
}

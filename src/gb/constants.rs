pub const OUTPUT_WIDTH: u8 = 160;
pub const OUTPUT_HEIGHT: u8 = 144;

pub const INTERRUPT_VBLANK_BIT: u8 = 0b00001;
pub const INTERRUPT_LCD_BIT: u8 = 0b00010;
pub const INTERRUPT_TIMER_BIT: u8 = 0b00100;
pub const INTERRUPT_SERIAL_BIT: u8 = 0b01000;
pub const INTERRUPT_JOYPAD_BIT: u8 = 0b10000;

pub const P1_ADDR: u16 = 0xFF00;
pub const DIV_ADDR: u16 = 0xFF04;
pub const INTERRUPTS_ADDR: u16 = 0xFF0F;
pub const INTERRUPT_ENABLE_ADDR: u16 = 0xFFFF;

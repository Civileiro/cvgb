use modular_bitfield::prelude::*;

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Key1 {
    pub switch_armed: bool,
    #[skip]
    __: B6,
    /// 0 = Normal speed, 1 = Double speed
    pub current_speed: bool,
}

impl Key1 {
    pub fn read(self) -> u8 {
        let data: u8 = self.into();
        data | 0b0111_1110
    }
    pub fn write(&mut self, data: u8) {
        *self = (data & 0x01).into()
    }
    pub fn switch_speed(&mut self) {
        debug_assert!(
            self.switch_armed(),
            "Speed switch should be armed before switching"
        );
        self.set_current_speed(!self.current_speed());
        self.set_switch_armed(false);
    }
}

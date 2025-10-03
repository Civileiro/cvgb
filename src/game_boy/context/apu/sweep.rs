#[derive(Debug, Default)]
pub struct Sweep {
    pace: u8,
    direction: SweepDirection,
    step: u8,

    timer: u8,
    enabled: bool,
    shadow_period: u16,
    pub period: u16,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SweepDirection(bool);

impl SweepDirection {
    pub const fn new(b: bool) -> Self {
        Self(b)
    }
    pub const fn bit(&self) -> u8 {
        self.0 as u8
    }
    pub const fn is_decreasing(&self) -> bool {
        self.0
    }
    pub const fn is_increasing(&self) -> bool {
        !self.is_decreasing()
    }
}

impl Sweep {
    pub fn read(&self) -> u8 {
        let mut res = 0x80;
        res |= self.pace << 4;
        res |= (self.direction.0 as u8) << 3;
        res |= self.step;
        res
    }
    pub fn write(&mut self, data: u8) {
        self.pace = (data >> 4) & 0b111;
        self.direction = SweepDirection((data >> 3) & 1 != 0);
        self.step = data & 0b111;
    }
    pub fn trigger(&mut self, disable_channel: &mut bool) {
        self.shadow_period = self.period;
        self.enabled = self.pace != 0 || self.step != 0;
        self.timer = Default::default();
        if self.step != 0 {
            self.step_period(disable_channel);
        }
    }
    pub fn tick(&mut self, disable_channel: &mut bool) {
        if !self.enabled {
            return;
        }
        self.timer -= 1;
        if self.timer == 0 {
            self.timer = if self.pace == 0 { 8 } else { self.pace };
            if self.pace != 0 {
                self.step_period(disable_channel);
            }
        }
    }
    fn step_period(&mut self, disable_channel: &mut bool) {
        let new_period = self.calculate_new_period();
        if new_period > 0x7FF {
            *disable_channel = true
        } else if new_period <= 0x7FF && self.step != 0 {
            self.shadow_period = new_period;
            self.period = new_period;
            if self.calculate_new_period() > 0x7FF {
                *disable_channel = true
            }
        }
    }
    fn calculate_new_period(&self) -> u16 {
        let addor = self.shadow_period >> self.step;
        if self.direction.is_increasing() {
            self.shadow_period + addor
        } else {
            self.shadow_period - addor
        }
    }
}

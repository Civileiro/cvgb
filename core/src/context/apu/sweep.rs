#[derive(Debug, Default)]
pub struct Sweep {
    pace: u8,
    direction: SweepDirection,
    shift: u8,

    timer: u8,
    enabled: bool,
    shadow_period: u16,
    has_calculated_while_negative: bool,
    pub period: u16,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SweepDirection(bool);

impl SweepDirection {
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
        res |= self.shift;
        res
    }
    pub fn write(&mut self, data: u8, disable_channel: &mut bool) {
        self.pace = (data >> 4) & 0b111;
        let old_direction = self.direction;
        self.direction = SweepDirection((data >> 3) & 1 != 0);
        // clearing sweep direction after at least one calculation disables the channel
        if self.enabled
            && self.has_calculated_while_negative
            && old_direction.is_decreasing()
            && self.direction.is_increasing()
        {
            *disable_channel = true
        }
        self.shift = data & 0b111;
    }
    pub fn timer_init(&self) -> u8 {
        if self.pace == 0 { 8 } else { self.pace }
    }
    pub fn trigger(&mut self, disable_channel: &mut bool) {
        self.shadow_period = self.period;
        self.enabled = self.pace != 0 || self.shift != 0;
        self.timer = self.timer_init();
        self.has_calculated_while_negative = false;
        if self.shift != 0 {
            self.calculate_new_period(disable_channel);
        }
    }
    pub fn tick(&mut self, disable_channel: &mut bool) {
        if !self.enabled {
            return;
        }
        self.timer -= 1;
        if self.timer == 0 {
            self.timer = self.timer_init();
            if self.pace != 0 {
                self.step_period(disable_channel);
            }
        }
    }
    fn step_period(&mut self, disable_channel: &mut bool) {
        let new_period = self.calculate_new_period(disable_channel);
        if new_period <= 0x7FF && self.shift != 0 {
            self.shadow_period = new_period;
            self.period = new_period;
            self.calculate_new_period(disable_channel);
        }
    }
    fn calculate_new_period(&mut self, disable_channel: &mut bool) -> u16 {
        self.has_calculated_while_negative = self.direction.is_decreasing();
        let addor = self.shadow_period >> self.shift;
        let new_period = if self.direction.is_increasing() {
            self.shadow_period + addor
        } else {
            self.shadow_period - addor
        };
        if new_period > 0x7FF {
            *disable_channel = true;
        }
        new_period
    }
}

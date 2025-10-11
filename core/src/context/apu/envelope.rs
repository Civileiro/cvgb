#[derive(Debug, Default, Clone, Copy)]
pub struct Envelope {
    volume: u8,
    direction: EnvelopeDirection,
    pace: u8,

    timer: u8,
    active: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeDirection(bool);

impl EnvelopeDirection {
    pub const fn new(b: bool) -> Self {
        Self(b)
    }
    pub const fn bit(&self) -> u8 {
        self.0 as u8
    }
    pub const fn is_decreasing(&self) -> bool {
        !self.is_increasing()
    }
    pub const fn is_increasing(&self) -> bool {
        self.0
    }
}
impl Envelope {
    pub fn read(&self) -> u8 {
        let mut res = 0;
        res |= self.volume << 4;
        res |= self.direction.bit() << 3;
        res |= self.pace;
        res
    }
    pub fn write(&mut self, data: u8) {
        self.volume = data >> 4;
        self.direction = EnvelopeDirection::new((data >> 3) & 1 != 0);
        self.pace = data & 0b111;
    }
    pub fn volume(&self) -> u8 {
        self.volume
    }
    pub fn init(&self) -> Self {
        let mut activated = *self;
        activated.active = activated.pace != 0 || activated.direction.is_increasing();
        activated.timer = activated.pace;
        activated
    }
    pub fn tick(&mut self) {
        if self.pace == 0 || !self.active {
            return;
        }
        self.timer -= 1;
        self.timer &= 0x07;
        if self.timer == 0 {
            self.timer = if self.pace == 0 { 8 } else { self.pace };
            if self.direction.is_increasing() {
                if self.volume < 0x0F {
                    self.volume += 1
                } else {
                    self.active = false
                }
            } else if self.volume > 0 {
                self.volume -= 1
            } else {
                self.active = false
            }
        }
    }
    pub fn dac_active(&self) -> bool {
        self.volume != 0 || self.direction.bit() != 0
    }
    pub fn zombie(&mut self, init: &Self) {
        let mut should_tick = init.pace != 0 && self.pace == 0 && self.active;
        let should_invert = init.direction != self.direction;
        if init.pace == 0
            && init.direction.is_increasing()
            && self.pace == 0
            && self.direction.is_increasing()
            && self.active
        {
            should_tick = true;
        }
        if should_invert {
            if init.direction.is_increasing() {
                if self.pace == 0 {
                    self.volume ^= 0xF;
                } else {
                    self.volume = 0xE - self.volume;
                    self.volume &= 0xF;
                }
                should_tick = false;
            } else {
                self.volume = 0x10 - self.volume;
                self.volume &= 0xF;
            }
        }
        if should_tick {
            if init.direction.is_increasing() {
                self.volume += 1;
            } else {
                self.volume -= 1;
            }
            self.volume &= 0xF;
        } else if init.pace == 0 {
            self.active = false
        }
        self.pace = init.pace;
        self.timer = init.pace;
        self.direction = init.direction;
    }
}

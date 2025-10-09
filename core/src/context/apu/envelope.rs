#[derive(Debug, Default, Clone, Copy)]
pub struct Envelope {
    volume: u8,
    direction: EnvelopeDirection,
    pace: u8,

    timer: u8,
}

#[derive(Debug, Default, Clone, Copy)]
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
    pub fn tick(&mut self) {
        if self.pace == 0 {
            return;
        }
        self.timer = (self.timer + 1) % self.pace;
        if self.timer == 0 {
            if self.direction.is_increasing() && self.volume < 0x0F {
                self.volume += 1;
            } else if self.direction.is_decreasing() && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
    pub fn dac_active(&self) -> bool {
        self.volume != 0 || self.direction.bit() != 0
    }
}

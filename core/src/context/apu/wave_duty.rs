#[derive(Debug, Default, Clone, Copy)]
pub enum WaveDuty {
    #[default]
    HalfQuarter = 0b00,
    OneQuarter = 0b01,
    TwoQuarters = 0b10,
    ThreeQuarters = 0b11,
}

impl WaveDuty {
    pub const fn from(data: u8) -> Option<Self> {
        let slf = match data {
            0 => Self::HalfQuarter,
            1 => Self::OneQuarter,
            2 => Self::TwoQuarters,
            3 => Self::ThreeQuarters,
            _ => return None,
        };
        Some(slf)
    }
}

use super::{envelope::Envelope, wave_duty::WaveDuty};

const CH2_LENGTH_TIMER_MAX: usize = 64;

#[derive(Debug, Default)]
pub struct Ch2 {
    active: bool,
    wave_duty: WaveDuty,
    sequence: u8,
    output: u8,
    initial_length_timer: usize,
    length_timer: usize,
    length_timer_enable: bool,
    period: u16,
    period_divider: u16,
    init_envelope: Envelope,
    active_envelope: Envelope,
}

impl Ch2 {
    const DUTY_TABLE: [u8; 4] = [0b1111_1110, 0b0111_1110, 0b0111_1000, 0b1000_0001];

    fn sample(&mut self) -> bool {
        if !self.active {
            return false;
        }
        let res = (Self::DUTY_TABLE[self.wave_duty as usize] >> self.sequence) & 1 != 0;
        self.sequence += 1;
        self.sequence &= 0b111;
        res
    }

    pub fn clock(&mut self) {
        self.period_divider += 1;
        if self.period_divider > 0x7FF {
            self.period_divider = self.period;
            self.output = if self.sample() && self.is_active() {
                self.active_envelope.volume()
            } else {
                0
            }
        }
    }
    pub fn get_output(&self) -> u8 {
        self.output
    }
    pub fn reset(&mut self) {
        *self = Self::default()
    }
    pub fn dac_active(&self) -> bool {
        self.init_envelope.dac_active()
    }
    pub fn trigger(&mut self) {
        if self.length_timer == CH2_LENGTH_TIMER_MAX {
            // TODO: check if it has to be reset to 0 instead
            self.length_timer = self.initial_length_timer;
        }
        self.active = true;
        self.period_divider = self.period;
        self.active_envelope = self.init_envelope;
    }
    pub fn is_active(&self) -> bool {
        self.active && self.dac_active()
    }
    pub fn length_timer_tick(&mut self) {
        if self.length_timer_enable && self.length_timer != CH2_LENGTH_TIMER_MAX {
            self.length_timer += 1;

            if self.length_timer == CH2_LENGTH_TIMER_MAX {
                self.active = false
            }
        }
    }
    pub fn envelope_tick(&mut self) {
        self.active_envelope.tick();
    }
    pub fn read_wave_duty(&self) -> u8 {
        (self.wave_duty as u8) << 6 | 0b0011_1111
    }
    pub fn write_wave_duty_and_length_timer(&mut self, data: u8) {
        self.wave_duty = WaveDuty::from(data >> 6).unwrap();
        self.initial_length_timer = (data & 0x3F) as usize;
    }
    pub fn read_volume_and_envelope(&self) -> u8 {
        self.init_envelope.read()
    }
    pub fn write_volume_and_envelope(&mut self, data: u8) {
        self.init_envelope.write(data);
        self.active &= self.dac_active();
    }
    pub fn write_period_low(&mut self, data: u8) {
        self.period &= 0xFF00;
        self.period |= data as u16;
    }
    pub fn read_period_high_and_control(&self) -> u8 {
        let mut res = 0b10111111;
        res |= (self.length_timer_enable as u8) << 6;
        res
    }
    pub fn write_period_high_and_control(&mut self, data: u8) {
        let trigger = (data >> 7) != 0;
        self.period &= 0x00FF;
        self.period |= ((data & 0b111) as u16) << 8;
        self.length_timer_enable = (data >> 6) & 1 != 0;
        if trigger {
            self.trigger();
        }
    }
}

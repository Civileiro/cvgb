use super::{envelope::Envelope, sample::SampleQueue, sweep::Sweep, wave_duty::WaveDuty};

const CH1_LENGTH_TIMER_MAX: usize = 64;

#[derive(Debug, Default)]
pub struct SquareChannel {
    active: bool,
    to_enable: bool,
    sweep: Sweep,
    wave_duty: WaveDuty,
    sequence: u8,
    initial_length_timer: usize,
    length_timer: usize,
    length_timer_enable: bool,
    next_tick_triggers_length: bool,
    period_divider: u16,
    init_envelope: Envelope,
    active_envelope: Envelope,
    sample_queue: SampleQueue<bool>,
}

impl SquareChannel {
    const DUTY_TABLE: [u8; 4] = [0b0100_0000, 0b1100_0000, 0b1111_0000, 0b0011_1111];

    fn sample(&mut self) -> bool {
        let res = (Self::DUTY_TABLE[self.wave_duty as usize] >> self.sequence) & 1 != 0;
        self.sequence += 1;
        self.sequence &= 0b111;
        res
    }

    pub fn clock(&mut self) {
        if !self.is_active() {
            return;
        }
        self.sample_queue.tick();
        self.period_divider += 1;
        if self.period_divider > 0x7FF {
            self.period_divider = self.sweep.period;
            let sample = self.sample();
            self.sample_queue.update_sample(sample);
        }
    }
    pub fn off_clock(&mut self) {
        if self.to_enable {
            self.active = true
        }
    }
    pub fn get_output(&self) -> u8 {
        if !self.is_active() || !self.sample_queue.get_sample() {
            0
        } else {
            self.active_envelope.volume()
        }
    }
    pub fn reset(&mut self) {
        *self = Self::default()
    }
    pub fn dac_active(&self) -> bool {
        self.init_envelope.dac_active()
    }
    pub fn trigger(&mut self) {
        if self.length_timer == CH1_LENGTH_TIMER_MAX {
            self.length_timer = if !self.next_tick_triggers_length && self.length_timer_enable {
                1
            } else {
                0
            };
        }
        let mut disable_channel = false;
        self.sweep.trigger(&mut disable_channel);
        self.to_enable = self.dac_active() && !disable_channel;
        self.period_divider = self.sweep.period;
        self.active_envelope = self.init_envelope;
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn length_timer_tick(&mut self) {
        if self.length_timer_enable && self.length_timer != CH1_LENGTH_TIMER_MAX {
            self.length_timer += 1;

            if self.length_timer == CH1_LENGTH_TIMER_MAX {
                self.active = false
            }
        }
    }
    pub fn signal_next_tick_length(&mut self, next_ticks: bool) {
        self.next_tick_triggers_length = next_ticks
    }
    pub fn set_length_timer_enable(&mut self, length_timer_enable: bool) {
        let prev_enable = self.length_timer_enable;
        self.length_timer_enable = length_timer_enable;
        // extra length tick occurs if next DIV-APU doesnt tick it
        // and it was just enabled, this does disable the channel if the
        // timer expires and it isnt being triggered in the same write
        if !self.next_tick_triggers_length && self.length_timer_enable && !prev_enable {
            self.length_timer_tick();
        }
    }
    pub fn sweep_tick(&mut self) {
        let mut disable_channel = false;
        self.sweep.tick(&mut disable_channel);
        if disable_channel {
            self.active = false
        }
    }
    pub fn envelope_tick(&mut self) {
        self.active_envelope.tick();
    }
    pub fn read_sweep(&self) -> u8 {
        self.sweep.read()
    }
    pub fn write_sweep(&mut self, data: u8) {
        let mut disable_channel = false;
        self.sweep.write(data, &mut disable_channel);
        if disable_channel {
            self.active = false
        }
    }
    pub fn read_wave_duty(&self) -> u8 {
        (self.wave_duty as u8) << 6 | 0b0011_1111
    }
    pub fn write_wave_duty_and_length_timer(&mut self, data: u8) {
        self.wave_duty = WaveDuty::from(data >> 6).unwrap();
        self.initial_length_timer = (data & 0x3F) as usize;
        self.length_timer = self.initial_length_timer;
    }
    pub fn read_volume_and_envelope(&self) -> u8 {
        self.init_envelope.read()
    }
    pub fn write_volume_and_envelope(&mut self, data: u8) {
        self.init_envelope.write(data);
        self.active &= self.dac_active();
    }
    pub fn write_period_low(&mut self, data: u8) {
        self.sweep.period &= 0xFF00;
        self.sweep.period |= data as u16;
    }
    pub fn read_period_high_and_control(&self) -> u8 {
        let mut res = 0b10111111;
        res |= (self.length_timer_enable as u8) << 6;
        res
    }
    pub fn write_period_high_and_control(&mut self, data: u8) {
        let trigger = (data >> 7) != 0;
        self.sweep.period &= 0x00FF;
        self.sweep.period |= ((data & 0b111) as u16) << 8;
        self.set_length_timer_enable((data >> 6) & 1 != 0);
        if trigger {
            self.trigger();
        }
    }
}

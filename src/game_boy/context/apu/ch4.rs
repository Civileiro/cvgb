use super::envelope::Envelope;

const CH4_LENGTH_TIMER_MAX: usize = 64;

#[derive(Debug, Default)]
pub struct Ch4 {
    active: bool,
    initial_length_timer: usize,
    length_timer: usize,
    length_timer_enable: bool,
    next_tick_triggers_length: bool,
    init_envelope: Envelope,
    active_envelope: Envelope,
    lfsr: Lfsr,
    output: u8,
}

#[derive(Debug, Default)]
struct Lfsr {
    pub clock_shift: u8,
    pub lfsr_width: bool,
    pub clock_divider: u8,
    clock: u16,
    buffer: u16,
}

impl Lfsr {
    pub fn reset_bits(&mut self) {
        self.buffer = 0;
    }
    fn clock_trigger_treshold(&self) -> u16 {
        let base = 16.0;
        let div = if self.clock_divider == 0 {
            0.5
        } else {
            self.clock_divider as f32
        };
        let shift = 2.0_f32.powf(self.clock_shift as f32);
        let res = base * div * shift;
        debug_assert!(res < u16::MAX as f32);
        res as u16
    }
    pub fn clock(&mut self) -> Option<bool> {
        self.clock += 1;
        if self.clock < self.clock_trigger_treshold() {
            return None;
        }
        self.clock = 0;

        let bit0 = self.buffer & 1 != 0;
        let bit1 = self.buffer & 2 != 0;
        let nxor_bit = !(bit0 ^ bit1);
        if nxor_bit {
            self.buffer |= 1 << 15;
            if self.lfsr_width {
                self.buffer |= 1 << 7;
            }
        } else {
            self.buffer &= !(1 << 15);
            if self.lfsr_width {
                self.buffer &= !(1 << 7);
            }
        };

        self.buffer >>= 1;
        Some(bit0)
    }
}

impl Ch4 {
    pub fn clock(&mut self) {
        if let Some(sample) = self.lfsr.clock() {
            self.output = if sample {
                self.active_envelope.volume()
            } else {
                0
            }
        }
    }
    pub fn reset(&mut self) {
        *self = Default::default()
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn dac_active(&self) -> bool {
        self.init_envelope.dac_active()
    }
    pub fn get_output(&self) -> u8 {
        self.output
    }
    fn trigger(&mut self) {
        if self.length_timer == CH4_LENGTH_TIMER_MAX {
            self.length_timer = if !self.next_tick_triggers_length && self.length_timer_enable {
                1
            } else {
                0
            };
        }
        self.active = self.dac_active();
        self.active_envelope = self.init_envelope;
    }
    pub fn length_timer_tick(&mut self) {
        if self.length_timer_enable && self.length_timer != CH4_LENGTH_TIMER_MAX {
            self.length_timer += 1;

            if self.length_timer == CH4_LENGTH_TIMER_MAX {
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
    pub fn envelope_tick(&mut self) {
        self.active_envelope.tick();
    }
    pub fn write_length_timer(&mut self, data: u8) {
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
    pub fn read_frequency_and_randomness(&self) -> u8 {
        let mut res = 0;
        res |= self.lfsr.clock_shift << 4;
        res |= (self.lfsr.lfsr_width as u8) << 3;
        res |= self.lfsr.clock_divider & 0b111;
        res
    }
    pub fn write_frequency_and_randomness(&mut self, data: u8) {
        self.lfsr.clock_shift = data >> 4;
        self.lfsr.lfsr_width = (data >> 3) & 1 != 0;
        self.lfsr.clock_divider = data & 0b111;
    }
    pub fn read_control(&self) -> u8 {
        (self.length_timer_enable as u8) << 6 | 0b1011_1111
    }
    pub fn write_control(&mut self, data: u8) {
        let trigger = (data >> 7) != 0;
        self.set_length_timer_enable((data >> 6) & 1 != 0);
        if trigger {
            self.trigger()
        }
    }
}

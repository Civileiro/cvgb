use super::sample::SampleQueue;

const CH3_LENGTH_TIMER_MAX: usize = 256;

#[derive(Debug, Default)]
pub struct WaveChannel {
    dac_enable: bool,
    active: bool,
    initial_length_timer: usize,
    length_timer: usize,
    length_timer_enable: bool,
    next_tick_triggers_length: bool,
    initial_volume: OutputVolume,
    active_volume: OutputVolume,
    period: u16,
    period_divider: u16,
    wave_ram: WaveRam,
    start_delay: bool,
    sample_queue: SampleQueue<u8>,
}

#[derive(Debug, Default, Clone, Copy)]
enum OutputVolume {
    #[default]
    Mute = 0b00,
    Full = 0b01,
    Half = 0b10,
    Quarter = 0b11,
}

impl OutputVolume {
    pub fn from(data: u8) -> Option<Self> {
        let slf = match data {
            0b00 => Self::Mute,
            0b01 => Self::Full,
            0b10 => Self::Half,
            0b11 => Self::Quarter,
            _ => return None,
        };
        Some(slf)
    }
    pub fn apply_volume(self, signal: u8) -> u8 {
        use OutputVolume::*;
        match self {
            Mute => 0,
            Full => signal,
            Half => signal >> 1,
            Quarter => signal >> 2,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct WaveRam {
    /// 32 nibbles
    ram: [u8; 16],
    /// index into the nibbles of the ram
    index: u8,
}

impl WaveRam {
    /// Read the nibble the index is pointing to
    fn read_index(&self) -> u8 {
        let byte = self.get_index_byte();
        let upper = self.index.is_multiple_of(2);
        if upper { byte >> 4 } else { byte & 0x0F }
    }
    pub fn get_index_byte(&self) -> u8 {
        self.ram[self.index as usize / 2]
    }
    pub fn get_index_byte_mut(&mut self) -> &mut u8 {
        &mut self.ram[self.index as usize / 2]
    }
    pub fn inc_index(&mut self) {
        self.index += 1;
        self.index &= 0x1F;
    }
    pub fn reset_index(&mut self) {
        self.index = 0;
    }
    pub fn read_byte(&self, index: u8) -> u8 {
        self.ram[index as usize]
    }
    pub fn write_byte(&mut self, index: u8, data: u8) {
        self.ram[index as usize] = data
    }
}

impl WaveChannel {
    pub fn clock(&mut self) {
        if self.start_delay {
            self.start_delay = false;
            return;
        }
        self.sample_queue.tick();
        self.period_divider += 1;
        if self.period_divider > 0x7FF {
            self.period_divider = self.period;
            self.wave_ram.inc_index();
            let sample = self.wave_ram.read_index();
            self.sample_queue.update_sample(sample);
        }
    }
    pub fn get_output(&self) -> u8 {
        self.active_volume
            .apply_volume(self.sample_queue.get_sample())
    }
    pub fn reset(&mut self) {
        let mut ram = self.wave_ram.clone();
        ram.reset_index();
        *self = Default::default();
        self.wave_ram = ram
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn dac_active(&self) -> bool {
        self.dac_enable
    }
    pub fn trigger(&mut self) {
        if self.length_timer == CH3_LENGTH_TIMER_MAX {
            self.length_timer = if !self.next_tick_triggers_length && self.length_timer_enable {
                1
            } else {
                0
            };
        }
        self.active = self.dac_active();
        self.start_delay = true;
        self.period_divider = self.period;
        self.active_volume = self.initial_volume;
        self.wave_ram.reset_index();
    }
    pub fn length_timer_tick(&mut self) {
        if self.length_timer_enable && self.length_timer != CH3_LENGTH_TIMER_MAX {
            self.length_timer += 1;

            if self.length_timer == CH3_LENGTH_TIMER_MAX {
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
    pub fn read_dac_enable(&self) -> u8 {
        (self.dac_enable as u8) << 7 | 0x7F
    }
    pub fn write_dac_enable(&mut self, data: u8) {
        self.dac_enable = data & 0x80 != 0;
        self.active &= self.dac_enable
    }
    pub fn write_length_timer(&mut self, data: u8) {
        self.initial_length_timer = data as usize;
        self.length_timer = self.initial_length_timer;
    }
    pub fn read_output_level(&self) -> u8 {
        (self.initial_volume as u8) << 5 | 0b1001_1111
    }
    pub fn write_output_level(&mut self, data: u8) {
        self.initial_volume = OutputVolume::from((data >> 5) & 0b11).unwrap();
    }
    pub fn write_period_low(&mut self, data: u8) {
        self.period &= 0xFF00;
        self.period |= data as u16;
    }
    pub fn read_period_high_and_control(&mut self) -> u8 {
        let mut res = 0b10111111;
        res |= (self.length_timer_enable as u8) << 6;
        res
    }
    pub fn write_period_high_and_control(&mut self, data: u8) {
        let trigger = (data >> 7) != 0;
        self.period &= 0x00FF;
        self.period |= ((data & 0b111) as u16) << 8;
        self.set_length_timer_enable((data >> 6) & 1 != 0);
        if trigger {
            self.trigger();
        }
    }
    pub fn read_wave_ram(&self, index: u8) -> u8 {
        if self.active {
            self.wave_ram.get_index_byte()
        } else {
            self.wave_ram.read_byte(index)
        }
    }
    pub fn write_wave_ram(&mut self, index: u8, data: u8) {
        if self.active {
            *self.wave_ram.get_index_byte_mut() = data;
        } else {
            self.wave_ram.write_byte(index, data);
        }
    }
}

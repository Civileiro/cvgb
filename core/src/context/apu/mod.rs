use modular_bitfield::prelude::*;
use noise_channel::NoiseChannel;
pub use output::{AudioBuffer, AudioOutput};
use square_channel::SquareChannel;
use wave_channel::WaveChannel;

use crate::context::IntoData;

mod delay;
mod envelope;
mod noise_channel;
mod output;
mod square_channel;
mod sweep;
mod wave_channel;

#[derive(Debug, Default)]
pub struct Apu {
    enabled: bool,
    frame_sequencer: FrameSequencer,
    div_bit: bool,
    time: usize,
    channels: Channels,
    vin_left: bool,
    vin_right: bool,
    volume_left: u8,
    volume_right: u8,
    capacitor_left: f32,
    capacitor_right: f32,
    ch1: SquareChannel,
    ch2: SquareChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,

    output_buffer: AudioBuffer,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Channels {
    pub ch1_right: bool,
    pub ch2_right: bool,
    pub ch3_right: bool,
    pub ch4_right: bool,
    pub ch1_left: bool,
    pub ch2_left: bool,
    pub ch3_left: bool,
    pub ch4_left: bool,
}

#[derive(Debug, Default)]
struct FrameSequencer {
    frame: Option<u8>,
    ignore_first: bool,
}

impl FrameSequencer {
    pub fn tick(&mut self) {
        if let Some(frame) = self.frame.as_mut() {
            *frame += 1;
            *frame &= 0b111;
        } else if self.ignore_first {
            self.ignore_first = false
        } else {
            self.frame = Some(0)
        }
    }
    pub fn length_tick(&self) -> bool {
        self.frame.is_some_and(|v| v.is_multiple_of(2))
    }
    pub fn sweep_tick(&self) -> bool {
        self.frame.is_some_and(|v| v == 2 || v == 6)
    }
    pub fn envelope_tick(&self) -> bool {
        self.frame.is_some_and(|v| v == 7)
    }
    pub fn power_on(&mut self, ignore_first: bool) {
        self.frame = None;
        self.ignore_first = ignore_first;
    }
}

impl Apu {
    pub fn cycle(&mut self) {
        self.time += 2;
        if self.time.is_multiple_of(2) {
            self.ch3.clock();
        }
        if self.time.is_multiple_of(4) {
            self.ch1.clock();
            self.ch2.clock();
            self.ch4.clock();
            let sample = self.calculate_output_sample();
            self.output_buffer.add_sample(sample);
        } else {
            self.ch1.off_clock();
            self.ch2.off_clock();
        }
    }
    pub fn calculate_output_sample(&mut self) -> [f32; 2] {
        let digital_to_analog = |dig: u8| 1.0 - (dig as f32) / 7.5;
        // Mix output of all channels
        let mixer_out = {
            let mut out = [0.0, 0.0];
            if self.ch1.dac_active() {
                let ch1_signal = digital_to_analog(self.ch1.get_output());
                if self.channels.ch1_left() {
                    out[0] += ch1_signal;
                }
                if self.channels.ch1_right() {
                    out[1] += ch1_signal;
                }
            }
            if self.ch2.dac_active() {
                let ch2_signal = digital_to_analog(self.ch2.get_output());
                if self.channels.ch2_left() {
                    out[0] += ch2_signal;
                }
                if self.channels.ch2_right() {
                    out[1] += ch2_signal;
                }
            }
            if self.ch3.dac_active() {
                let ch3_signal = digital_to_analog(self.ch3.get_output());
                if self.channels.ch3_left() {
                    out[0] += ch3_signal;
                }
                if self.channels.ch3_right() {
                    out[1] += ch3_signal;
                }
            }
            if self.ch4.dac_active() {
                let ch4_signal = digital_to_analog(self.ch4.get_output());
                if self.channels.ch4_left() {
                    out[0] += ch4_signal;
                }
                if self.channels.ch4_right() {
                    out[1] += ch4_signal;
                }
            }
            // Normalize signal from [-4, 4] to [-1, 1]
            out[0] /= 4.0;
            out[1] /= 4.0;
            out
        };
        // Apply stereo volume settings
        let volume_out = {
            let mut out = mixer_out;
            out[0] *= (self.volume_left + 1) as f32 / 8.0;
            out[1] *= (self.volume_right + 1) as f32 / 8.0;
            out
        };
        // volume_out
        // Apply High-pass filter
        let filter_coefficient = {
            // TODO: configurable cutoff
            let cutoff = 50.0;
            let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
            let t = 1.0 / crate::APU_SAMPLE_RATE as f32;
            rc / (rc + t)
        };
        let mut out = [0.0, 0.0];
        out[0] = volume_out[0] - self.capacitor_left;
        self.capacitor_left = volume_out[0] - out[0] * filter_coefficient;
        out[1] = volume_out[1] - self.capacitor_right;
        self.capacitor_right = volume_out[1] - out[1] * filter_coefficient;
        out
    }
    pub fn get_audio_output(&mut self) -> AudioOutput {
        self.output_buffer.get_output()
    }
    fn div_apu_tick(&mut self) {
        self.frame_sequencer.tick();
        if self.frame_sequencer.length_tick() {
            self.ch1.length_timer_tick();
            self.ch2.length_timer_tick();
            self.ch3.length_timer_tick();
            self.ch4.length_timer_tick();

            self.ch1.signal_next_tick_length(false);
            self.ch2.signal_next_tick_length(false);
            self.ch3.signal_next_tick_length(false);
            self.ch4.signal_next_tick_length(false);
        } else {
            self.ch1.signal_next_tick_length(true);
            self.ch2.signal_next_tick_length(true);
            self.ch3.signal_next_tick_length(true);
            self.ch4.signal_next_tick_length(true);
        }
        if self.frame_sequencer.sweep_tick() {
            self.ch1.sweep_tick();
        }
        if self.frame_sequencer.envelope_tick() {
            self.ch1.envelope_tick();
            self.ch2.envelope_tick();
            self.ch4.envelope_tick();
        }
    }
    pub fn set_div_bit(&mut self, bit: bool) {
        let old_bit = self.div_bit;
        self.div_bit = bit;
        if old_bit && !self.div_bit {
            self.div_apu_tick();
        }
    }
    /// Read Channel 1 Sweep
    pub fn cycle_nr10_read(&mut self) -> u8 {
        self.cycle();
        self.ch1.read_sweep()
    }
    /// Write Channel 1 Sweep
    pub fn cycle_nr10_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch1.write_sweep(data)
        }
    }
    /// Read Channel 1 Duty Cycle
    pub fn cycle_nr11_read(&mut self) -> u8 {
        self.cycle();
        self.ch1.read_wave_duty()
    }
    /// Read Channel 1 Duty Cycle & Length Timer
    pub fn cycle_nr11_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch1.write_wave_duty_and_length_timer(data);
        }
    }
    /// Read Channel 1 Volume & Envelope
    pub fn cycle_nr12_read(&mut self) -> u8 {
        self.cycle();
        self.ch1.read_volume_and_envelope()
    }
    /// Write Channel 1 Volume & Envelope
    pub fn cycle_nr12_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch1.write_volume_and_envelope(data);
        }
    }
    /// Read Channel 1 NOTHING (its write-only)
    pub fn cycle_nr13_read(&mut self) -> u8 {
        ().into_data()
    }
    /// Write Channel 1 Period Low
    pub fn cycle_nr13_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch1.write_period_low(data);
        }
    }
    /// Read Channel 1 Period High & Control
    pub fn cycle_nr14_read(&mut self) -> u8 {
        self.cycle();
        self.ch1.read_period_high_and_control()
    }
    /// Write Channel 1 Period High & Control
    pub fn cycle_nr14_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch1.write_period_high_and_control(data);
        }
    }
    /// Read Channel 2 Duty Cycle
    pub fn cycle_nr21_read(&mut self) -> u8 {
        self.cycle();
        self.ch2.read_wave_duty()
    }
    /// Read Channel 2 Duty Cycle & Length Timer
    pub fn cycle_nr21_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch2.write_wave_duty_and_length_timer(data);
        }
    }
    /// Read Channel 2 Volume & Envelope
    pub fn cycle_nr22_read(&mut self) -> u8 {
        self.cycle();
        self.ch2.read_volume_and_envelope()
    }
    /// Write Channel 2 Volume & Envelope
    pub fn cycle_nr22_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch2.write_volume_and_envelope(data);
        }
    }
    /// Read Channel 2 NOTHING (its write-only)
    pub fn cycle_nr23_read(&mut self) -> u8 {
        ().into_data()
    }
    /// Write Channel 2 Period Low
    pub fn cycle_nr23_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch2.write_period_low(data);
        }
    }
    /// Read Channel 2 Period High & Control
    pub fn cycle_nr24_read(&mut self) -> u8 {
        self.cycle();
        self.ch2.read_period_high_and_control()
    }
    /// Write Channel 2 Period High & Control
    pub fn cycle_nr24_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch2.write_period_high_and_control(data);
        }
    }
    /// Read Channel 3 DAC Enable
    pub fn cycle_nr30_read(&mut self) -> u8 {
        self.cycle();
        self.ch3.read_dac_enable()
    }
    /// Write Channel 3 DAC Enable
    pub fn cycle_nr30_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch3.write_dac_enable(data);
        }
    }
    /// Write Channel 3 Length Timer
    pub fn cycle_nr31_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch3.write_length_timer(data);
        }
    }
    /// Read Channel 3 Output Level
    pub fn cycle_nr32_read(&mut self) -> u8 {
        self.cycle();
        self.ch3.read_output_level()
    }
    /// Write Channel 3 Output Level
    pub fn cycle_nr32_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch3.write_output_level(data);
        }
    }
    /// Write Channel 3 Period Low
    pub fn cycle_nr33_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch3.write_period_low(data);
        }
    }
    /// Read Channel 3 Period High & Control
    pub fn cycle_nr34_read(&mut self) -> u8 {
        self.cycle();
        self.ch3.read_period_high_and_control()
    }
    /// Write Channel 3 Period High & Control
    pub fn cycle_nr34_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch3.write_period_high_and_control(data);
        }
    }
    /// Read Channel 3 Wave Pattern RAM
    pub fn cycle_pattern_ram_read(&mut self, index: u8) -> u8 {
        self.cycle();
        self.ch3.read_wave_ram(index)
    }
    /// Write Channel 3 Period High & Control
    pub fn cycle_pattern_ram_write(&mut self, index: u8, data: u8) {
        self.cycle();
        self.ch3.write_wave_ram(index, data);
    }
    /// Write Channel 4 Length Timer
    pub fn cycle_nr41_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch4.write_length_timer(data);
        }
    }
    /// Read Channel 4 Volume & Envelope
    pub fn cycle_nr42_read(&mut self) -> u8 {
        self.cycle();
        self.ch4.read_volume_and_envelope()
    }
    /// Write Channel 4 Volume & Envelope
    pub fn cycle_nr42_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch4.write_volume_and_envelope(data);
        }
    }
    /// Read Channel 4 Frequency & Randomness
    pub fn cycle_nr43_read(&mut self) -> u8 {
        self.cycle();
        self.ch4.read_frequency_and_randomness()
    }
    /// Write Channel 4 Frequency & Randomness
    pub fn cycle_nr43_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch4.write_frequency_and_randomness(data);
        }
    }
    /// Read Channel 4 Control
    pub fn cycle_nr44_read(&mut self) -> u8 {
        self.cycle();
        self.ch4.read_control()
    }
    /// Write Channel 4 Control
    pub fn cycle_nr44_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.ch4.write_control(data);
        }
    }
    /// Read APU Master Volume & VIN panning
    pub fn cycle_nr50_read(&mut self) -> u8 {
        self.cycle();
        let mut res = 0;
        res |= (self.vin_left as u8) << 7;
        res |= self.volume_left << 4;
        res |= (self.vin_right as u8) << 3;
        res |= self.volume_right;
        res
    }
    /// Write APU Master Volume & VIN panning
    pub fn cycle_nr50_write(&mut self, data: u8) {
        if self.enabled {
            self.vin_left = data & 0x80 != 0;
            self.volume_left = (data >> 4) & 0b111;
            self.vin_right = data & 0x08 != 0;
            self.volume_right = data & 0b111;
        }
        self.cycle();
    }
    fn reset_nr50(&mut self) {
        self.vin_left = false;
        self.volume_left = 0;
        self.vin_right = false;
        self.volume_right = 0;
    }
    /// Read APU Sound Panning
    pub fn cycle_nr51_read(&mut self) -> u8 {
        self.cycle();
        self.channels.into()
    }
    /// Write APU Sound Panning
    pub fn cycle_nr51_write(&mut self, data: u8) {
        self.cycle();
        if self.enabled {
            self.channels = data.into();
        }
    }
    fn reset_nr51(&mut self) {
        self.channels = 0.into();
    }
    /// Read Audio Master Control
    pub fn cycle_nr52_read(&mut self) -> u8 {
        self.cycle();
        let mut res = 0b0111_0000;
        res |= (self.enabled as u8) << 7;
        res |= (self.ch4.is_active() as u8) << 3;
        res |= (self.ch3.is_active() as u8) << 2;
        res |= (self.ch2.is_active() as u8) << 1;
        res |= self.ch1.is_active() as u8;
        res
    }
    /// Write Audio Master Control
    pub fn cycle_nr52_write(&mut self, data: u8) {
        self.cycle();
        let old_enabled = self.enabled;
        self.enabled = data & 0x80 != 0;
        if !self.enabled {
            self.ch4.reset();
            self.ch3.reset();
            self.ch2.reset();
            self.ch1.reset();
            self.reset_nr50();
            self.reset_nr51();
        } else if !old_enabled && self.enabled {
            let ignore_first = self.div_bit;
            self.frame_sequencer.power_on(ignore_first);
            self.time = 0;
        }
    }
    pub fn cycle_pcm12_read(&mut self) -> u8 {
        self.cycle();
        self.ch2.get_output() << 4 | self.ch1.get_output()
    }
    pub fn cycle_pcm34_read(&mut self) -> u8 {
        self.cycle();
        self.ch4.get_output() << 4 | self.ch3.get_output()
    }
}

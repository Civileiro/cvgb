use modular_bitfield::prelude::*;

#[derive(Debug, Default)]
pub struct Timer {
    sys_clock: u16,
    /// Timer Counter
    tima: u8,
    // Timer Modulo
    tma: u8,
    // Timer Control
    tac: Tac,
    overflowed: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
struct Tac {
    pub clock_select: B2,
    pub enable: bool,
    #[skip]
    __: B5,
}

impl Timer {
    pub fn div(&self) -> u8 {
        (self.sys_clock >> 6) as u8
    }
    pub fn reset_div(&mut self) {
        self.sys_clock = 0
    }
    pub fn frequency_mask(&self) -> u16 {
        match self.tac.clock_select() {
            0b00 => 0b10000000,
            0b01 => 0b00000010,
            0b10 => 0b00001000,
            0b11 => 0b00100000,
            _ => unreachable!(),
        }
    }
    fn tick(&mut self) {
        let (tima, overflow) = self.tima.overflowing_add(1);
        self.tima = tima;
        self.overflowed = overflow;
    }
    fn clock(&mut self, ctx: &mut impl TimerContext) {
        if self.overflowed {
            self.overflowed = false;
            self.tima = self.tma;
            ctx.signal_timer_interrupt();
        }
        self.sys_clock = self.sys_clock.wrapping_add(1);
    }
    fn selected_bit(&self) -> bool {
        self.sys_clock & self.frequency_mask() != 0
    }
    fn div_apu_bit(&self, ctx: &mut impl TimerContext) -> bool {
        let mask = if ctx.is_double_speed() {
            1 << 11
        } else {
            1 << 10
        };
        self.sys_clock & mask != 0
    }
    fn watch_falling_edges<C: TimerContext, T>(
        &mut self,
        ctx: &mut C,
        f: impl FnOnce(&mut Self, &mut C) -> T,
    ) -> T {
        let prev_bit = self.selected_bit();
        let res = f(self, ctx);
        let new_bit = self.selected_bit();
        if self.tac.enable() && prev_bit && !new_bit {
            self.tick();
        }
        let div_apu_bit = self.div_apu_bit(ctx);
        ctx.signal_div_apu_bit(div_apu_bit);
        res
    }

    pub fn cycle(&mut self, ctx: &mut impl TimerContext) {
        self.watch_falling_edges(ctx, |slf, ctx| slf.clock(ctx))
    }
    pub fn cycle_read_div(&mut self, ctx: &mut impl TimerContext) -> u8 {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            slf.div()
        })
    }
    pub fn cycle_write_div(&mut self, ctx: &mut impl TimerContext, _data: u8) {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            slf.sys_clock = 0;
        })
    }
    pub fn cycle_read_tima(&mut self, ctx: &mut impl TimerContext) -> u8 {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            slf.tima
        })
    }
    pub fn cycle_write_tima(&mut self, ctx: &mut impl TimerContext, data: u8) {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            // Writing to TIMA during an overflow cycle causes it to be ignored
            slf.overflowed = false;
            slf.tima = data
        })
    }
    pub fn cycle_read_tma(&mut self, ctx: &mut impl TimerContext) -> u8 {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            slf.tma
        })
    }
    pub fn cycle_write_tma(&mut self, ctx: &mut impl TimerContext, data: u8) {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.tma = data;
            slf.clock(ctx);
        })
    }
    pub fn cycle_read_tac(&mut self, ctx: &mut impl TimerContext) -> u8 {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            slf.tac.into()
        })
    }
    pub fn cycle_write_tac(&mut self, ctx: &mut impl TimerContext, data: u8) {
        self.watch_falling_edges(ctx, |slf, ctx| {
            slf.clock(ctx);
            slf.tac = data.into();
        })
    }
}

pub trait TimerContext {
    fn signal_timer_interrupt(&mut self);
    fn is_double_speed(&self) -> bool;
    fn signal_div_apu_bit(&mut self, bit: bool);
}

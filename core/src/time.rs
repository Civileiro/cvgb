use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

const BASE_SYSTEM_CLOCK: u64 = 4_194_304;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemTime {
    /// Always advances at 4.194304 MHz
    /// Double speed mode is simulated by making non-affected
    /// components take double the time
    base_master_clock_cycles: u64,
}

impl SystemTime {
    pub fn new() -> Self {
        Self {
            base_master_clock_cycles: 0,
        }
    }
    pub fn from_master_clocks(master_clocks: u64) -> Self {
        Self {
            base_master_clock_cycles: master_clocks,
        }
    }
    pub fn from_system_clocks(system_clocks: u64) -> Self {
        Self {
            base_master_clock_cycles: system_clocks * 4,
        }
    }
    pub fn from_seconds(seconds: f64) -> Self {
        let clocks = BASE_SYSTEM_CLOCK as f64 * seconds;
        Self {
            base_master_clock_cycles: clocks.floor() as u64,
        }
    }
    pub fn seconds(&self) -> f64 {
        self.base_master_clock_cycles as f64 / BASE_SYSTEM_CLOCK as f64
    }
    pub fn cycles(&self) -> u64 {
        self.base_master_clock_cycles
    }
}

impl Display for SystemTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SystemTime({} clocks)", self.base_master_clock_cycles)
    }
}

impl Add<Self> for SystemTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            base_master_clock_cycles: self.base_master_clock_cycles + rhs.base_master_clock_cycles,
        }
    }
}

impl AddAssign<Self> for SystemTime {
    fn add_assign(&mut self, rhs: Self) {
        self.base_master_clock_cycles += rhs.base_master_clock_cycles
    }
}

impl Sub<Self> for SystemTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            base_master_clock_cycles: self.base_master_clock_cycles - rhs.base_master_clock_cycles,
        }
    }
}

impl SubAssign<Self> for SystemTime {
    fn sub_assign(&mut self, rhs: Self) {
        self.base_master_clock_cycles -= rhs.base_master_clock_cycles
    }
}

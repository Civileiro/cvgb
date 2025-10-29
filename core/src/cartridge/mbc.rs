use super::CartridgeMemory;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Controller {
    fn clock(&mut self) {}
    fn read(&self, memory: &CartridgeMemory, addr: u16) -> u8;
    fn write(&mut self, memory: &mut CartridgeMemory, addr: u16, data: u8);
}

const OUT_OF_RANGE_MESSAGE: &str = "Address is out of range for cartridge access";

#[enum_dispatch(Controller)]
#[derive(Debug)]
pub enum MemoryBankController {
    MBC1,
    // MBC2,
    MBC3,
    // MBC5,
    // MBC6,
    // MBC7,
    // MMM01,
    // HuC1,
    // HuC3,
}

impl MemoryBankController {
    pub fn from_cartridge_type(cartridge_type: u8) -> Option<Self> {
        if cartridge_type == 0 {
            return None;
        }
        let slf = match cartridge_type {
            0x00 => return None,
            0x01..=0x03 => Self::mbc1(),
            // 0x05 | 0x06 => Self::mbc2(),
            // 0x0B => Self::MMM01,
            // 0x0C => Self::MMM01,
            // 0x0D => Self::MMM01,
            0x0F => Self::mbc3(),
            0x10 => Self::mbc3(),
            0x11 => Self::mbc3(),
            0x12 => Self::mbc3(),
            0x13 => Self::mbc3(),
            // 0x19 => Self::MBC5,
            // 0x1A => Self::MBC5,
            // 0x1B => Self::MBC5,
            // 0x1C => Self::MBC5,
            // 0x1D => Self::MBC5,
            // 0x1E => Self::MBC5,
            // 0x20 => Self::MBC6,
            // 0x22 => Self::MBC7,
            // 0xFE => Self::HuC3,
            // 0xFF => Self::HuC1,
            _ => {
                log::warn!("Unknown cartridge type: {cartridge_type:02x}. Using no MBC");
                return None;
            }
        };
        Some(slf)
    }
    pub fn mbc1() -> Self {
        Self::MBC1(MBC1::default())
    }
    pub fn mbc3() -> Self {
        Self::MBC3(MBC3::default())
    }
}

impl<T: Controller> Controller for Option<T> {
    fn read(&self, memory: &CartridgeMemory, addr: u16) -> u8 {
        if let Some(mbc) = self.as_ref() {
            return mbc.read(memory, addr);
        }
        match addr {
            0x0000..0x8000 => memory.read_rom(addr.into()),
            0xA000..0xC000 => memory.read_ram(addr & 0x1FFF),
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }

    fn write(&mut self, memory: &mut CartridgeMemory, addr: u16, data: u8) {
        if let Some(mbc) = self.as_mut() {
            return mbc.write(memory, addr, data);
        }
        match addr {
            0x0000..0x8000 => (),
            0xA000..0xC000 => memory.write_ram(addr & 0x1FFF, data),
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }
}

#[derive(Debug)]
pub struct MBC1 {
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    banking_mode: bool,
}

impl Default for MBC1 {
    fn default() -> Self {
        Self {
            ram_enable: Default::default(),
            rom_bank_number: 1,
            ram_bank_number: Default::default(),
            banking_mode: Default::default(),
        }
    }
}

impl Controller for MBC1 {
    fn read(&self, memory: &CartridgeMemory, addr: u16) -> u8 {
        match addr {
            0x0000..0x4000 => {
                if self.banking_mode {
                    let addr = ((addr & 0x3FFF) as u32) | ((self.ram_bank_number as u32) << 19);
                    memory.read_rom(addr)
                } else {
                    memory.read_rom(addr.into())
                }
            }
            0x4000..0x8000 => {
                let addr = ((addr & 0x3FFF) as u32)
                    | ((self.rom_bank_number as u32) << 14)
                    | ((self.ram_bank_number as u32) << 19);
                memory.read_rom(addr)
            }
            0xA000..0xC000 => {
                let addr = addr & 0x1FFF;
                if !self.ram_enable {
                    0xFF
                } else if self.banking_mode {
                    let addr = addr | ((self.ram_bank_number as u16) << 13);
                    memory.read_ram(addr)
                } else {
                    memory.read_ram(addr)
                }
            }
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }
    fn write(&mut self, memory: &mut CartridgeMemory, addr: u16, data: u8) {
        match addr {
            0x0000..0x2000 => {
                self.ram_enable = (data & 0x0F) == 0x0A;
            }
            0x2000..0x4000 => {
                self.rom_bank_number = (data & 0x1F).max(1);
            }
            0x4000..0x6000 => {
                self.ram_bank_number = data & 0x03;
            }
            0x6000..0x8000 => {
                self.banking_mode = data & 1 != 0;
            }
            0xA000..0xC000 => {
                let addr = addr & 0x1FFF;
                if !self.ram_enable {
                    return;
                }
                if self.banking_mode {
                    let addr = addr | ((self.ram_bank_number as u16) << 13);
                    memory.write_ram(addr, data);
                } else {
                    memory.write_ram(addr, data);
                }
            }
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }
}

#[derive(Debug, Default)]
pub struct MBC3 {
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    rtc: RealTimeClock,
    latch: RealTimeClock,
}

#[derive(Debug, Default, Clone, Copy)]
struct RealTimeClock {
    clocks: usize,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days: u8,
    high: u8,
}

impl Controller for MBC3 {
    fn clock(&mut self) {
        const CLOCKS_PER_SECOND: usize = crate::BASE_CPU_FREQUENCY;
        const CLOCKS_PER_MINUTE: usize = CLOCKS_PER_SECOND * 60;
        const CLOCKS_PER_HOUR: usize = CLOCKS_PER_MINUTE * 60;
        const CLOCKS_PER_DAY: usize = CLOCKS_PER_HOUR * 24;
        self.rtc.clocks += 4;
        if self.rtc.clocks >= CLOCKS_PER_SECOND {
            self.rtc.clocks -= CLOCKS_PER_SECOND;
            self.rtc.seconds += 1;
        } else {
            return;
        }
        if self.rtc.seconds == 60 {
            self.rtc.seconds = 0;
            self.rtc.minutes += 1;
        }
        if self.rtc.minutes == 60 {
            self.rtc.minutes = 0;
            self.rtc.hours += 1;
        }
        let overflow: bool;
        if self.rtc.hours == 24 {
            self.rtc.hours = 0;
            let (days, c) = self.rtc.days.overflowing_add(1);
            self.rtc.days = days;
            overflow = c;
        } else {
            return;
        }
        if overflow {
            if self.rtc.high & 0x01 != 0 {
                self.rtc.high |= 0x80;
            }
            self.rtc.high ^= 0x01;
        }
    }
    fn read(&self, memory: &CartridgeMemory, addr: u16) -> u8 {
        match addr {
            0x0000..0x4000 => memory.read_rom(addr.into()),
            0x4000..0x8000 => {
                let addr = ((addr & 0x3FFF) as u32) | ((self.rom_bank_number.max(1) as u32) << 14);
                memory.read_rom(addr)
            }
            0xA000..0xC000 => {
                let addr = addr - 0xA000;
                if !self.ram_enable {
                    0xFF
                } else if self.ram_bank_number & 0xF8 == 0 {
                    let addr = addr | ((self.ram_bank_number as u16) << 13);
                    memory.read_ram(addr)
                } else {
                    let rtc_reg = self.ram_bank_number & 0x07;
                    match rtc_reg {
                        0 => self.latch.seconds,
                        1 => self.latch.minutes,
                        2 => self.latch.hours,
                        3 => self.latch.days,
                        4 => self.latch.high,
                        _ => 0xFF,
                    }
                }
            }
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }

    fn write(&mut self, memory: &mut CartridgeMemory, addr: u16, data: u8) {
        match addr {
            0x0000..0x2000 => {
                self.ram_enable = (data & 0x0F) == 0x0A;
            }
            0x2000..0x4000 => {
                self.rom_bank_number = data & 0x7F;
            }
            0x4000..0x6000 => {
                self.ram_bank_number = data & 0x0F;
            }
            0x6000..0x8000 => {
                self.latch = self.rtc;
            }
            0xA000..0xC000 => {
                let addr = addr - 0xA000;
                if !self.ram_enable {
                    return;
                }
                if self.ram_bank_number & 0xF8 == 0 {
                    let addr = addr | ((self.ram_bank_number as u16) << 13);
                    memory.write_ram(addr, data);
                } else {
                    let rtc_reg = self.ram_bank_number & 0x07;
                    match rtc_reg {
                        0 => self.latch.seconds = data,
                        1 => self.latch.minutes = data,
                        2 => self.latch.hours = data,
                        3 => self.latch.days = data,
                        4 => self.latch.high = data,
                        _ => (),
                    }
                }
            }
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }
}

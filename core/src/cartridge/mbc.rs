use super::CartridgeMemory;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Controller {
    fn read(&self, memory: &CartridgeMemory, addr: u16) -> u8;
    fn write(&mut self, memory: &mut CartridgeMemory, addr: u16, data: u8);
}

const OUT_OF_RANGE_MESSAGE: &str = "Address is out of range for cartridge access";

#[enum_dispatch(Controller)]
#[derive(Debug)]
pub enum MemoryBankController {
    MBC1,
    // MBC2,
    // MBC3,
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
            0x01..=0x03 => Self::mbc1(),
            // 0x05 | 0x06 => Self::mbc2(),
            // 0x0B => Self::MMM01,
            // 0x0C => Self::MMM01,
            // 0x0D => Self::MMM01,
            // 0x0F => Self::MBC3,
            // 0x10 => Self::MBC3,
            // 0x11 => Self::MBC3,
            // 0x12 => Self::MBC3,
            // 0x13 => Self::MBC3,
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
}

impl<T: Controller> Controller for Option<T> {
    fn read(&self, memory: &CartridgeMemory, addr: u16) -> u8 {
        if let Some(mbc) = self.as_ref() {
            return mbc.read(memory, addr);
        }
        match addr {
            0x0000..0x8000 => memory.read_rom(addr as u32),
            0xA000..0xC000 => memory.read_ram(addr - 0xA000),
            _ => unimplemented!("{OUT_OF_RANGE_MESSAGE}"),
        }
    }

    fn write(&mut self, memory: &mut CartridgeMemory, addr: u16, data: u8) {
        if let Some(mbc) = self.as_mut() {
            return mbc.write(memory, addr, data);
        }
        match addr {
            0x0000..0x8000 => (),
            0xA000..0xC000 => memory.write_ram(addr - 0xA000, data),
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
                    let addr = (addr as u32) | ((self.ram_bank_number as u32) << 19);
                    memory.rom[addr as usize]
                } else {
                    memory.rom[addr as usize]
                }
            }
            0x4000..0x8000 => {
                let addr = ((addr & 0x3FFF) as u32)
                    | ((self.rom_bank_number as u32) << 14)
                    | ((self.ram_bank_number as u32) << 19);
                memory.read_rom(addr)
            }
            0xA000..0xC000 => {
                let addr = addr & 0x9FFF;
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
                let addr = addr - 0xA000;
                if !self.banking_mode {
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

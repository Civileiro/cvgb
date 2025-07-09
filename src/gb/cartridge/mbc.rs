use crate::gb::{Addressable, bus::AddressingError};

use super::Cartridge;

#[derive(Debug, Default)]
pub enum MemoryBankController {
    #[default]
    None,
    MBC1 {
        ram_enable: bool,
        rom_bank_number: u8,
        ram_bank_number: u8,
        banking_mode: bool,
    },
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    MMM01,
    HuC1,
    HuC3,
}

impl MemoryBankController {
    pub fn none() -> Self {
        Self::None
    }
    pub fn mbc1() -> Self {
        Self::MBC1 {
            ram_enable: false,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: false,
        }
    }
    pub fn read(&self, cartridge: &Cartridge, addr: u16) -> Result<u8, AddressingError> {
        match self {
            MemoryBankController::None => match addr {
                0x0000..=0x7FFF => cartridge
                    .rom
                    .get(addr as usize)
                    .copied()
                    .ok_or(AddressingError::Unmapped),
                0xA000..=0xBFFF => cartridge
                    .ram
                    .as_ref()
                    .map(|ram| ram.read(addr))
                    .unwrap_or(Err(AddressingError::Unmapped)),
                _ => Err(AddressingError::Unmapped),
            },
            MemoryBankController::MBC1 {
                ram_enable,
                rom_bank_number,
                ram_bank_number,
                banking_mode,
            } => match addr {
                0x0000..=0x3FFF => {
                    if *banking_mode {
                        let addr = (addr as u32) | ((*ram_bank_number as u32) << 19);
                        cartridge.read_rom(addr)
                    } else {
                        cartridge.read_rom(addr.into())
                    }
                }
                0x4000..=0x7FFF => {
                    let addr = ((addr - 0x4000) as u32)
                        | ((*rom_bank_number as u32) << 14)
                        | ((*ram_bank_number as u32) << 19);
                    cartridge.read_rom(addr)
                }
                0xA000..=0xBFFF => {
                    let addr = addr - 0xA000;
                    if !*ram_enable {
                        Err(AddressingError::Unmapped)
                    } else if *banking_mode {
                        let addr = addr | ((*ram_bank_number as u16) << 13);
                        cartridge
                            .read_ram(addr)
                            .unwrap_or(Err(AddressingError::Unmapped))
                    } else {
                        cartridge
                            .read_ram(addr)
                            .unwrap_or(Err(AddressingError::Unmapped))
                    }
                }
                _ => Err(AddressingError::Unmapped),
            },
            MemoryBankController::MBC2 => todo!(),
            MemoryBankController::MBC3 => todo!(),
            MemoryBankController::MBC5 => todo!(),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
            MemoryBankController::MMM01 => todo!(),
            MemoryBankController::HuC1 => todo!(),
            MemoryBankController::HuC3 => todo!(),
        }
    }
    pub fn write(
        &mut self,
        cartridge: &Cartridge,
        addr: u16,
        data: u8,
    ) -> Result<(), AddressingError> {
        match self {
            MemoryBankController::None => match addr {
                0x0000..=0x7FFF | 0xA000..=0xBFFF => Ok(()),
                _ => Err(AddressingError::Unmapped),
            },
            MemoryBankController::MBC1 {
                ram_enable,
                rom_bank_number,
                ram_bank_number,
                banking_mode,
            } => match addr {
                0x0000..=0x1FFF => {
                    *ram_enable = (data & 0x0F) == 0x0A;
                    Ok(())
                }
                0x2000..=0x3FFF => {
                    *rom_bank_number = data & 0x1F;
                    if *rom_bank_number == 0x00 {
                        *rom_bank_number = 0x01
                    };
                    Ok(())
                }
                0x4000..=0x5FFF => {
                    *ram_bank_number = data & 0x3;
                    Ok(())
                }
                0x6000..=0x7FFF => {
                    *banking_mode = (data & 1) == 1;
                    Ok(())
                }
                0xA000..=0xBFFF => {
                    let addr = addr - 0xA000;
                    if !*ram_enable {
                        Err(AddressingError::Unmapped)
                    } else if *banking_mode {
                        let addr = addr | ((*ram_bank_number as u16) << 13);
                        cartridge
                            .write_ram(addr, data)
                            .unwrap_or(Err(AddressingError::Unmapped))
                    } else {
                        cartridge
                            .write_ram(addr, data)
                            .unwrap_or(Err(AddressingError::Unmapped))
                    }
                }
                _ => Err(AddressingError::Unmapped),
            },
            MemoryBankController::MBC2 => todo!(),
            MemoryBankController::MBC3 => todo!(),
            MemoryBankController::MBC5 => todo!(),
            MemoryBankController::MBC6 => todo!(),
            MemoryBankController::MBC7 => todo!(),
            MemoryBankController::MMM01 => todo!(),
            MemoryBankController::HuC1 => todo!(),
            MemoryBankController::HuC3 => todo!(),
        }
    }
}

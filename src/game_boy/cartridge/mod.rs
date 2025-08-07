mod mbc;

use compact_str::CompactString;
use mbc::MemoryBankController;
use thiserror::Error;

pub type Rom = Box<[u8]>;

#[derive(Debug)]
pub struct Cartridge {
    rom: Rom,
    ram: Box<[u8]>,
}

#[derive(Debug, Error)]
pub enum CartridgeParseError {
    #[error("unknown cartridge type {0:02x}")]
    UnknownCartridgeType(u8),
}

impl Cartridge {
    pub fn from_rom(rom: Rom) -> Result<Self, CartridgeParseError> {
        let read = |addr| rom.get(addr).copied().unwrap_or(0x00);
        let read_ascii = |start, end| {
            let mut ascii = CompactString::new("");
            for addr in start..=end {
                let c = read(addr);
                if c == 0x00 {
                    break;
                }
                let Some(char) = char::from_u32(c as u32) else {
                    break;
                };
                ascii.push(char);
            }
            ascii
        };
        log::info!("Loading cartridge...");
        let title = read_ascii(0x134, 0x013E);
        log::info!("title = \"{title}\"");
        let manufacturer_code = read_ascii(0x13F, 0x0142);
        log::info!("manufacturer code = \"{manufacturer_code}\"");
        let cgb_flag = read(0x0143);
        log::info!("cgb_flag = {cgb_flag:02x}");
        let new_licensee_code = read_ascii(0x0144, 0x0145);
        log::info!("new licensee code = \"{new_licensee_code}\"");
        let sgb_flag = read(0x0146);
        log::info!("sgb_flag = {sgb_flag:02x}");
        let cartridge_type = read(0x0147);
        log::info!("cartridge type = {cartridge_type:02x}");
        let rom_size_byte = read(0x0148).min(0x08);
        log::info!("rom size byte = {rom_size_byte:02x}");
        let rom_size = (32 << 10) * (1 << rom_size_byte);
        log::info!("rom size = {rom_size:x}");
        log::info!("actual size = {:x}", rom.len());
        let ram_size = read(0x0149);
        log::info!("ram size bytes = {ram_size:02x}");
        let destination_code = read(0x014A);
        log::info!("destination code = {destination_code:02x}");
        let old_licensee_code = read(0x014B);
        log::info!("old licensee code = {old_licensee_code:02x}");

        let version_number = read(0x014C);
        log::info!("version number = {version_number:02x}");
        let checksum = read(0x014D);
        log::info!("checksum = {checksum:02x}");
        let global_checksum = ((read(0x014E) as u16) << 8) + (read(0x014F) as u16);
        log::info!("global checksum = {global_checksum:02x}");
        let ram_bank_count = match ram_size {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => 0,
        };
        let (mbc, has_ram, battery) = match cartridge_type {
            0x00 => (MemoryBankController::none(), false, false),
            0x01 => (MemoryBankController::mbc1(), false, false),
            0x02 => (MemoryBankController::mbc1(), true, false),
            0x03 => (MemoryBankController::mbc1(), true, true),
            0x05 => (MemoryBankController::MBC2, false, false),
            0x06 => (MemoryBankController::MBC2, false, true),
            // 0x08 => (MemoryBankController::None, false, false),
            // 0x09 => (MemoryBankController::None, false, false),
            0x0B => (MemoryBankController::MMM01, false, false),
            0x0C => (MemoryBankController::MMM01, true, false),
            0x0D => (MemoryBankController::MMM01, true, true),
            0x0F => (MemoryBankController::MBC3, false, true),
            0x10 => (MemoryBankController::MBC3, true, true),
            0x11 => (MemoryBankController::MBC3, false, false),
            0x12 => (MemoryBankController::MBC3, true, false),
            0x13 => (MemoryBankController::MBC3, true, true),
            0x19 => (MemoryBankController::MBC5, false, false),
            0x1A => (MemoryBankController::MBC5, true, false),
            0x1B => (MemoryBankController::MBC5, true, true),
            0x1C => (MemoryBankController::MBC5, false, false),
            0x1D => (MemoryBankController::MBC5, true, false),
            0x1E => (MemoryBankController::MBC5, true, true),
            0x20 => (MemoryBankController::MBC6, false, false),
            0x22 => (MemoryBankController::MBC7, true, true),
            0xFE => (MemoryBankController::HuC3, false, false),
            0xFF => (MemoryBankController::HuC1, true, true),
            _ => return Err(CartridgeParseError::UnknownCartridgeType(cartridge_type)),
        };
        let ram = {
            // 16 KiB / bank
            let ram_size = if has_ram {
                ram_bank_count * (16 << 10)
            } else {
                0
            };
            let ram_bytes = vec![0; ram_size].into_boxed_slice();
            ram_bytes
        };
        Ok(Self { rom, ram })
    }
}

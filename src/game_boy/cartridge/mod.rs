mod mbc;

use compact_str::CompactString;
use mbc::{Controller, MemoryBankController};
use thiserror::Error;

pub type Rom = Box<[u8]>;

#[derive(Debug)]
pub struct Cartridge {
    header: CartridgeHeader,
    memory: CartridgeMemory,
    mbc: Option<MemoryBankController>,
}

#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: CompactString,
    pub manufacturer_code: CompactString,
    pub cgb_flag: u8,
    pub new_licensee_code: CompactString,
    pub sgb_flag: u8,
    pub cartridge_type: u8,
    pub rom_size_byte: u8,
    pub ram_size: u8,
    pub destination_code: u8,
    pub old_licensee_code: u8,
    pub version_number: u8,
    pub checksum: u8,
    pub global_checksum: u16,
}

impl CartridgeHeader {
    fn from_rom(rom: &Rom) -> Option<Self> {
        let read = |addr| rom.get(addr).copied();
        let read_ascii = |start, end| {
            let mut ascii = CompactString::new("");
            for addr in start..=end {
                let c = read(addr)?;
                if c == 0x00 {
                    break;
                }
                let Some(char) = char::from_u32(c as u32) else {
                    break;
                };
                ascii.push(char);
            }
            Some(ascii)
        };
        log::info!("Reading cartridge header...");
        let title = read_ascii(0x134, 0x013E)?;
        log::info!("title = \"{title}\"");
        let manufacturer_code = read_ascii(0x13F, 0x0142)?;
        log::info!("manufacturer code = \"{manufacturer_code}\"");
        let cgb_flag = read(0x0143)?;
        log::info!("cgb_flag = {cgb_flag:02x}");
        let new_licensee_code = read_ascii(0x0144, 0x0145)?;
        log::info!("new licensee code = \"{new_licensee_code}\"");
        let sgb_flag = read(0x0146)?;
        log::info!("sgb_flag = {sgb_flag:02x}");
        let cartridge_type = read(0x0147)?;
        log::info!("cartridge type = {cartridge_type:02x}");
        let rom_size_byte = read(0x0148)?.min(0x08);
        log::info!("rom size byte = {rom_size_byte:02x}");
        let rom_size = (32 << 10) * (1 << rom_size_byte);
        log::info!("rom size = {rom_size:x}");
        log::info!("actual size = {:x}", rom.len());
        let ram_size = read(0x0149)?;
        log::info!("ram size bytes = {ram_size:02x}");
        let destination_code = read(0x014A)?;
        log::info!("destination code = {destination_code:02x}");
        let old_licensee_code = read(0x014B)?;
        log::info!("old licensee code = {old_licensee_code:02x}");
        let version_number = read(0x014C)?;
        log::info!("version number = {version_number:02x}");
        let checksum = read(0x014D)?;
        log::info!("checksum = {checksum:02x}");
        let global_checksum = ((read(0x014E)? as u16) << 8) + (read(0x014F)? as u16);
        log::info!("global checksum = {global_checksum:02x}");

        Some(Self {
            title,
            manufacturer_code,
            cgb_flag,
            new_licensee_code,
            sgb_flag,
            cartridge_type,
            rom_size_byte,
            ram_size,
            destination_code,
            old_licensee_code,
            version_number,
            checksum,
            global_checksum,
        })
    }
    fn rom_size(&self) -> usize {
        (32 << 10) * (1 << self.rom_size_byte)
    }
    fn ram_bank_count(&self) -> usize {
        match self.ram_size {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => 0,
        }
    }
}

#[derive(Debug)]
pub struct CartridgeMemory {
    rom: Rom,
    ram: Box<[u8]>,
}

impl CartridgeMemory {
    fn rom_mask(&self) -> u32 {
        debug_assert_eq!(self.rom.len().count_ones(), 1);
        self.rom.len() as u32 - 1
    }
    pub fn read_rom(&self, addr: u32) -> u8 {
        self.rom[(addr & self.rom_mask()) as usize]
    }
    pub fn read_ram(&self, addr: u16) -> u8 {
        if self.ram.is_empty() {
            return 0xFF;
        }
        self.ram[addr as usize % self.ram.len()]
    }
    pub fn write_ram(&mut self, addr: u16, data: u8) {
        if self.ram.is_empty() {
            return;
        }
        self.ram[addr as usize % self.ram.len()] = data
    }
}

#[derive(Debug, Error)]
pub enum CartridgeParseError {
    #[error("Error while reading cartridge header")]
    HeaderError,
}

impl Cartridge {
    pub fn from_rom(rom: Rom) -> Result<Self, CartridgeParseError> {
        let header = CartridgeHeader::from_rom(&rom).ok_or(CartridgeParseError::HeaderError)?;
        let mbc = MemoryBankController::from_cartridge_type(header.cartridge_type);
        let ram = {
            // 16 KiB / bank
            let ram_size = header.ram_bank_count() * (16 << 10);

            vec![0; ram_size].into_boxed_slice()
        };
        Ok(Self {
            header,
            memory: CartridgeMemory { rom, ram },
            mbc,
        })
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(&self.memory, addr)
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        self.mbc.write(&mut self.memory, addr, data);
    }
}

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
}

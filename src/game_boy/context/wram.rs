const WRAM_ADDR_START: u16 = 0xC000;
const WRAM_ADDR_END: u16 = 0xFE00;
const WRAM_BANK_SIZE: u16 = 0x1000;
const WRAM_SIZE: u16 = WRAM_BANK_SIZE * 8;
const WRAM_ADDR_USABLE_BITS: u16 = 0x1FFF;

#[derive(Debug)]
pub struct WorkRam {
    bank: u8,
    ram: Box<[u8]>,
}

impl Default for WorkRam {
    fn default() -> Self {
        Self {
            bank: Default::default(),
            ram: vec![0; WRAM_SIZE.into()].into_boxed_slice(),
        }
    }
}

impl WorkRam {
    /// Read WRAM. Address is expected to be relative to global gb memory.
    /// Address start at [`0xC000`] and are mirrored up to [`u16::MAX`]
    pub fn read(&self, addr: u16) -> u8 {
        self.ram[self.to_wram_addr(addr) as usize]
    }
    /// Write WRAM. Address is expected to be relative to global gb memory
    /// Address start at [`0xC000`] and are mirrored up to [`u16::MAX`]
    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[self.to_wram_addr(addr) as usize] = data
    }
    pub fn read_svbk(&self) -> u8 {
        self.bank
    }
    pub fn write_svbk(&mut self, data: u8) {
        self.bank = data & 0b111;
    }
    fn to_wram_addr(&self, addr: u16) -> u16 {
        debug_assert!(
            matches!(addr, WRAM_ADDR_START..WRAM_ADDR_END),
            "Invalid address for WRAM: {addr:04x}"
        );
        let rel_addr = addr & WRAM_ADDR_USABLE_BITS;
        if (addr >> 13) == 1 {
            ((self.bank as u16).min(1) << 13) | rel_addr
        } else {
            rel_addr
        }
    }
}

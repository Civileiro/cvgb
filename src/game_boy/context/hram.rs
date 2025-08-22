const HRAM_START_ADDR: u16 = 0xFF80;
const HRAM_END_ADDR: u16 = 0xFFFF;
const HRAM_SIZE: u16 = HRAM_END_ADDR - HRAM_START_ADDR;

#[derive(Debug)]
pub struct HighRam {
    memory: Box<[u8]>,
}

impl Default for HighRam {
    fn default() -> Self {
        Self {
            memory: vec![0; HRAM_SIZE.into()].into_boxed_slice(),
        }
    }
}

impl HighRam {
    /// Read HRAM. Address is expected to be relative to global gb memory.
    pub fn read(&self, addr: u16) -> u8 {
        self.memory[Self::to_hram_addr(addr) as usize]
    }
    /// Write HRAM. Address is expected to be relative to global gb memory.
    pub fn write(&mut self, addr: u16, data: u8) {
        self.memory[Self::to_hram_addr(addr) as usize] = data
    }

    fn to_hram_addr(addr: u16) -> u16 {
        addr & !0xFF80
    }
}

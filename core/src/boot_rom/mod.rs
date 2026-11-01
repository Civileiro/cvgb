#[derive(Debug)]
pub struct BootRom {
    rom: &'static [u8],
    enabled: bool,
}

impl BootRom {
    pub fn new(enabled: bool) -> Self {
        let rom_bytes = include_bytes!("cgb.bin");
        Self {
            rom: rom_bytes,
            enabled,
        }
    }
    pub fn disable(&mut self) {
        self.enabled = false
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
}

impl Default for BootRom {
    fn default() -> Self {
        Self::new(true)
    }
}

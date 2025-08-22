#[derive(Debug, Default)]
pub struct WorkRam {}

impl WorkRam {
    /// Read WRAM. Address is expected to be relative to global gb memory.
    /// Address start at [`0xC000`] and are mirrored up to [`u16::MAX`]
    pub fn read(&self, addr: u16) -> u8 {
        todo!()
    }
    /// Write WRAM. Address is expected to be relative to global gb memory
    /// Address start at [`0xC000`] and are mirrored up to [`u16::MAX`]
    pub fn write(&mut self, addr: u16, data: u8) {
        todo!()
    }
    pub fn read_svbk(&self) -> u8 {
        todo!()
    }
    pub fn write_svbk(&mut self, data: u8) {
        todo!()
    }
}

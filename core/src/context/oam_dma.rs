use crate::Cartridge;

use super::{MemoryRegion, ppu::Ppu, wram::WorkRam};

#[derive(Debug, Default)]
pub struct OamDma {
    dma: u8,
    fetched_data: Option<u8>,
    start: u16,
    curr: u16,
    end: u16,
}

impl OamDma {
    pub fn cycle_read_dma(&mut self, ppu: &mut Ppu, cartridge: &Cartridge, wram: &WorkRam) -> u8 {
        self.cycle(ppu, cartridge, wram);
        self.dma
    }
    fn read_dma(&self) -> u8 {
        self.dma
    }
    pub fn cycle_write_dma(
        &mut self,
        ppu: &mut Ppu,
        cartridge: &Cartridge,
        wram: &WorkRam,
        data: u8,
    ) {
        self.cycle(ppu, cartridge, wram);
        self.write_dma(data);
    }
    fn write_dma(&mut self, data: u8) {
        self.dma = data;
        let start = (data as u16) << 8;
        self.start = start;
        self.curr = start;
        self.end = start + 0xA0;
        self.fetched_data = None;
    }
    pub fn start_address(&self) -> u16 {
        (self.dma as u16) << 8
    }
    pub fn curr_oam_addr(&self) -> u16 {
        0xFE00 | (self.curr & 0xFF)
    }
    pub fn is_active(&self) -> bool {
        self.curr < self.end || self.fetched_data.is_some()
    }
    fn oam_dma_addr_region(addr: u16) -> MemoryRegion {
        match addr {
            0x0000..0x8000 | 0xA000..0xC000 => MemoryRegion::Cartridge,
            0x8000..0xA000 => MemoryRegion::VRam,
            0xC000.. => MemoryRegion::WRam,
        }
    }
    pub fn active_memory_region(&self) -> Option<MemoryRegion> {
        if !self.is_active() {
            return None;
        }
        Some(Self::oam_dma_addr_region(self.start))
    }
    pub fn cycle(&mut self, ppu: &mut Ppu, cartridge: &Cartridge, wram: &WorkRam) {
        if let Some(data) = self.fetched_data.take() {
            // TODO: lock oam from further writes as reads in this cycle
            ppu.write_oam(self.curr_oam_addr(), data);
            self.curr += 1;
        }
        if self.curr < self.end {
            let fetched_data = match Self::oam_dma_addr_region(self.curr) {
                MemoryRegion::Cartridge => cartridge.read(self.curr),
                MemoryRegion::VRam => ppu.read_vram(self.curr),
                MemoryRegion::WRam => wram.read(self.curr),
                _ => unreachable!("OAM DMA can't access other memory regions"),
            };
            self.fetched_data = Some(fetched_data)
        }
    }
}

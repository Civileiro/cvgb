use modular_bitfield::prelude::*;

use crate::game_boy::Cartridge;

use super::{
    IntoData,
    ppu::{Ppu, VRAM_START_ADDR},
    wram::WorkRam,
};

const HDMA_BATCH_SIZE: u8 = 16;

#[derive(Debug, Default)]
pub struct Hdma {
    hdma_src_high: u8,
    hdma_src_low: u8,
    hdma_dst_high: u8,
    hdma_dst_low: u8,

    /// if the registers have been updated since last transfer
    updated: bool,
    transfer: TransferProcedure,
}

#[derive(Debug, Default)]
struct TransferProcedure {
    src: u16,
    dst: u16,
    curr: u16,
    length: u16,

    /// false = GDMA, true = HDMA
    mode: bool,
    /// if the transfer was stopped manually
    stopped: bool,
    hdma_batch: u8,
}

impl TransferProcedure {
    fn finished(&self) -> bool {
        self.stopped || self.completed()
    }
    fn completed(&self) -> bool {
        self.remaining_length() == 0
    }
    fn remaining_length(&self) -> u16 {
        self.length - self.curr
    }
    fn paused(&self) -> bool {
        !self.finished() && self.mode && self.hdma_batch == 0
    }
    fn read_addr(&self) -> u16 {
        self.src + self.curr
    }
    fn write_addr(&self) -> u16 {
        VRAM_START_ADDR + self.dst + self.curr
    }
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct Hdma5 {
    blocks: B7,
    mode: bool,
}

impl Hdma {
    pub fn write_hdma_src_high(&mut self, data: u8) {
        self.hdma_src_high = data;
        self.updated = true;
    }
    pub fn write_hdma_src_low(&mut self, data: u8) {
        self.hdma_src_low = data & 0xF0;
        self.updated = true;
    }
    pub fn write_hdma_dst_high(&mut self, data: u8) {
        self.hdma_dst_high = data & 0x1F;
        self.updated = true;
    }
    pub fn write_hdma_dst_low(&mut self, data: u8) {
        self.hdma_dst_low = data & 0xF0;
        self.updated = true;
    }
    pub fn read_hdma5(&self) -> u8 {
        if self.transfer.finished() {
            ().into_data()
        } else {
            let rem_bytes = self.transfer.remaining_length();
            let rem_blocks = (rem_bytes / HDMA_BATCH_SIZE as u16) as u8 - 1;
            ((self.transfer.stopped as u8) << 7) | rem_blocks
        }
    }
    pub fn write_hdma5(&mut self, ppu: &Ppu, data: u8) {
        let mut data = Hdma5::from(data);
        if !self.transfer.finished() {
            if data.mode() {
                // reset transfer
                self.updated = true;
                data.set_mode(self.transfer.mode);
            } else {
                // stop transfer
                self.transfer.stopped = true;
                return;
            }
        }
        self.init_transfer(
            u16::from_be_bytes([self.hdma_src_high, self.hdma_src_low]),
            u16::from_be_bytes([self.hdma_dst_high, self.hdma_dst_low]),
            data.blocks(),
            data.mode(),
            if ppu.read_mode().is_hblank() { 16 } else { 0 },
        );
    }
    fn init_transfer(&mut self, src: u16, dst: u16, blocks: u8, mode: bool, init_batch: u8) {
        // if a copy is started without changing the source/destination addresses
        // it will continue from the last addresses of the previous copy
        if self.transfer.src == src
            && self.transfer.dst == dst
            && !self.updated
            && self.transfer.length != 0
            && self.transfer.completed()
        {
            self.transfer = TransferProcedure {
                src: self.transfer.src + self.transfer.length,
                dst: self.transfer.dst + self.transfer.length,
                curr: 0,
                length: (blocks as u16 + 1) * HDMA_BATCH_SIZE as u16,
                mode,
                stopped: false,
                hdma_batch: init_batch,
            }
        } else {
            self.transfer = TransferProcedure {
                src,
                dst,
                curr: 0,
                length: (blocks as u16 + 1) * HDMA_BATCH_SIZE as u16,
                mode,
                stopped: false,
                hdma_batch: init_batch,
            }
        }
        self.updated = false
    }
    /// Progresses current tranfer by [`num_bytes`] bytes if theres one, returns true if the system should
    /// continue transfering
    pub fn transfer(
        &mut self,
        ppu: &mut Ppu,
        cartridge: &Cartridge,
        wram: &WorkRam,
        num_bytes: u8,
    ) -> bool {
        for _ in 0..num_bytes {
            if self.transfer.finished() || self.transfer.paused() {
                break;
            }
            self.transfer1(ppu, cartridge, wram);
        }
        !(self.transfer.finished() || self.transfer.paused())
    }
    fn transfer1(&mut self, ppu: &mut Ppu, cartridge: &Cartridge, wram: &WorkRam) {
        let transfer = &mut self.transfer;
        if transfer.finished() || transfer.paused() {
            return;
        }
        let read_addr = transfer.read_addr();
        let data = match read_addr {
            0x0000..0x8000 | 0xA000..0xC000 => cartridge.read(read_addr),
            0xC000..0xE000 => wram.read(read_addr),
            0xE000.. => cartridge.read(read_addr - 0x4000),
            _ => ().into_data(),
        };
        let write_addr = transfer.write_addr();
        ppu.write_vram(write_addr, data);

        transfer.curr += 1;
        if transfer.mode {
            transfer.hdma_batch -= 1;
        }
    }
    pub fn signal_mode_0(&mut self) {
        self.transfer.hdma_batch = HDMA_BATCH_SIZE
    }
}

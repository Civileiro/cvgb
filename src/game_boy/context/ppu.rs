// Most things here are placeholders

use std::fmt::Debug;

use enum_assoc::Assoc;
use modular_bitfield::prelude::*;

use crate::game_boy::events::Events;

use super::{IntoData, interrupts::InterruptFlags};

const VRAM_START_ADDR: u16 = 0x8000;
const VRAM_END_ADDR: u16 = 0xA000;
const VRAM_BANK_SIZE: u16 = VRAM_END_ADDR - VRAM_START_ADDR;

const OAM_START_ADDR: u16 = 0xFE00;
const OAM_END_ADDR: u16 = 0xFEA0;
const OAM_SIZE: u16 = OAM_END_ADDR - OAM_START_ADDR;

const PALLETE_MEM_SIZE: usize = 64;

#[derive(Default)]
pub struct VideoBuffer {}

#[derive(Default)]
pub struct Display {
    buffer: VideoBuffer,
}

impl Display {
    pub fn get_video_buffer(&self) -> &VideoBuffer {
        &self.buffer
    }
}

pub struct Ppu {
    display: Display,
    vram: Box<[u8]>,
    vram_bank: u8,
    oam: Box<[u8]>,
    oam_lock: bool,
    bcg_palette_mem: [u8; PALLETE_MEM_SIZE],
    obj_palette_mem: [u8; PALLETE_MEM_SIZE],
    lcdc: Lcdc,
    stat: Stat,
}

#[derive(Debug, Clone, Copy, Assoc, Specifier)]
#[func(pub fn duration(&self) -> u16)]
pub enum Mode {
    #[assoc(duration = 20)]
    OAMScan = 2,
    #[assoc(duration = 43)]
    Drawing = 3,
    #[assoc(duration = 51)]
    HBlank = 0,
    #[assoc(duration = 114)]
    VBlank = 1,
}

impl Mode {
    pub fn is_hblank(&self) -> bool {
        matches!(self, Self::HBlank)
    }
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct ObjectAttributes {
    cgb_palette: B3,
    bank: B1,
    dmg_palette: B1,
    x_flip: bool,
    y_flip: bool,
    not_priority: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub struct BackgroundAttributes {
    color_palette: B3,
    bank: B1,
    #[skip]
    __: B1,
    x_flip: bool,
    y_flip: bool,
    priority: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Lcdc {
    bg_enable_or_priority: bool,
    obj_enable: bool,
    obj_size: bool,
    bg_tile_map_area: bool,
    bg_tile_addr_mode: bool,
    window_enable: bool,
    window_tile_map_area: bool,
    pub ppu_enable: bool,
}

#[bitfield(bits = 8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Stat {
    ppu_mode: Mode,
    lyc_eq_ly: bool,
    mode0_int_select: bool,
    mode1_int_select: bool,
    mode2_int_select: bool,
    lyc_int_select: bool,
    #[skip]
    __: B1,
}

impl Debug for Ppu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ppu")
            .field("vram_bank", &self.vram_bank)
            .finish_non_exhaustive()
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self {
            display: Default::default(),
            vram: vec![0; (VRAM_BANK_SIZE as usize) * 2].into_boxed_slice(),
            vram_bank: Default::default(),
            oam: vec![0; OAM_SIZE as usize].into_boxed_slice(),
            oam_lock: false,
            bcg_palette_mem: [0; PALLETE_MEM_SIZE],
            obj_palette_mem: [0; PALLETE_MEM_SIZE],
            lcdc: Default::default(),
            stat: Default::default(),
        }
    }
}

pub trait PpuContext {
    fn signal_lcd_interrupt(&mut self);
    fn signal_frame_ready(&mut self);
    fn is_double_speed(&self) -> bool;
}

impl Ppu {
    pub fn cycle(&mut self, ctx: &mut impl PpuContext) {
        todo!()
    }
    /// Read VRAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn cycle_read_vram(&self, ctx: &mut impl PpuContext, addr: u16) -> u8 {
        todo!()
        // self.vram[self.to_vram_address(addr) as usize]
    }
    /// Write VRAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Writes nothing if the PPU is in the wrong mode
    pub fn cycle_write_vram(&mut self, ctx: &mut impl PpuContext, addr: u16, data: u8) {
        todo!()
        // self.vram[self.to_vram_address(addr) as usize] = data
    }
    /// Read OAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn cycle_read_oam(&self, ctx: &mut impl PpuContext, addr: u16) -> u8 {
        todo!()
        // self.oam[self.to_oam_address(addr) as usize]
    }
    /// Write OAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Writes nothing if the PPU is in the wrong mode
    pub fn cycle_write_oam(&mut self, ctx: &mut impl PpuContext, addr: u16, data: u8) {
        todo!()
        // self.oam[self.to_oam_address(addr) as usize] = data
    }
    /// Read VRAM. Address is expected to be relative to global gb memory
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[self.to_vram_address(addr) as usize]
    }
    /// Write VRAM. Address is expected to be relative to global gb memory
    pub fn write_vram(&mut self, addr: u16, data: u8) {
        self.vram[self.to_vram_address(addr) as usize] = data
    }
    /// Read OAM. Address is expected to be relative to global gb memory
    pub fn read_oam(&self, addr: u16) -> u8 {
        self.oam[self.to_oam_address(addr) as usize]
    }
    /// Write OAM. Address is expected to be relative to global gb memory
    pub fn write_oam(&mut self, addr: u16, data: u8) {
        self.oam[self.to_oam_address(addr) as usize] = data
    }
    pub fn read_lcdc(&self) -> u8 {
        self.lcdc.into()
    }
    pub fn write_lcdc(&mut self, data: u8) {
        self.lcdc = Lcdc::from(data)
    }
    pub fn read_stat(&self) -> u8 {
        self.stat.into()
    }
    pub fn write_stat(&mut self, data: u8) {
        let curr: u8 = self.stat.into();
        let new = data & 0b1111_1100 | curr & 0b11;
        self.stat = Stat::from(new)
    }
    fn to_vram_address(&self, addr: u16) -> u16 {
        debug_assert!(
            matches!(addr, VRAM_START_ADDR..VRAM_END_ADDR),
            "Invalid address for VRAM: {addr:04x}"
        );
        let abs_addr = addr - VRAM_START_ADDR;
        let offset = self.vram_bank as u16 * 0x2000;
        abs_addr + offset
    }
    fn to_oam_address(&self, addr: u16) -> u16 {
        debug_assert!(
            matches!(addr, OAM_START_ADDR..OAM_END_ADDR),
            "Invalid address for OAM: {addr:04x}"
        );
        addr - OAM_START_ADDR
    }
    pub fn read_mode(&self) -> Mode {
        if self.lcdc.ppu_enable() {
            self.stat.ppu_mode()
        } else {
            Mode::HBlank
        }
    }
    pub fn set_oam_lock(&mut self, block_oam: bool) {
        self.oam_lock = block_oam
    }
    pub fn get_video_buffer(&self) -> &VideoBuffer {
        self.display.get_video_buffer()
    }
}

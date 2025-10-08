use std::fmt::Debug;

use arrayvec::ArrayVec;
use enum_assoc::Assoc;
use modular_bitfield::prelude::*;

use crate::game_boy::{WINDOW_HEIGHT, WINDOW_WIDTH};

use super::IntoData;

pub const VRAM_START_ADDR: u16 = 0x8000;
pub const VRAM_END_ADDR: u16 = 0xA000;
pub const VRAM_BANK_SIZE: u16 = VRAM_END_ADDR - VRAM_START_ADDR;

pub const OAM_START_ADDR: u16 = 0xFE00;
pub const OAM_END_ADDR: u16 = 0xFEA0;
pub const OAM_SIZE: u16 = OAM_END_ADDR - OAM_START_ADDR;

pub const PALLETE_MEM_SIZE: usize =
    Palette::NUM_PALETTES * Palette::COLORS_PER_PALETTE * Palette::BYTES_PER_COLOR;
pub const PALETTE_MEM_BITS: u8 = PALLETE_MEM_SIZE as u8 - 1;

pub const WINDOW_BUFFER_SIZE: usize = WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize;

pub struct Ppu {
    video_buffer: VideoBuffer,
    /// The time left on the current mode
    mode_timer: i32,
    /// 16 Kib Video RAM (8 KiB x 2 Banks)
    vram: Box<[u8]>,
    /// Video RAM Bank Select
    vram_bank: bool,
    /// 160 Byte Object Attribute Memory
    oam: Box<[u8]>,
    /// OAM Access Lock
    oam_lock: bool,
    /// Background Palette Memory
    bcg_palette_mem: [u8; PALLETE_MEM_SIZE],
    /// Object Palette Memory
    obj_palette_mem: [u8; PALLETE_MEM_SIZE],
    /// Base palette used for DMG compatibility
    dmg_palette: Option<Palette>,
    /// LCD Control
    lcdc: Lcdc,
    /// LCD Status
    stat: Stat,
    /// Scroll Y
    pub scy: u8,
    /// Scroll X
    pub scx: u8,
    /// LCD Y Coord
    ly: u8,
    /// LY Compare
    pub lyc: u8,
    /// Background Palette Data (Non-CGB Mode)
    pub bgp: u8,
    /// Object Palette Data 0 (Non-CGB Mode)
    pub obp0: u8,
    /// Object Palette Data 1 (Non-CGB Mode)
    pub obp1: u8,
    /// Window Y Position
    pub wy: u8,
    /// Window X  Position
    pub wx: u8,
    /// Object Priority Mode
    pub opri: bool,
    /// Background Color Palette Index
    pub bgpi: u8,
    /// Object Color Palette Index
    pub ocpi: u8,
}

#[derive(Debug, Clone, Copy, Assoc, Specifier)]
#[func(pub fn duration(&self) -> u16)]
pub enum Mode {
    #[assoc(duration = 80)]
    OAMScan = 2,
    #[assoc(duration = 172)]
    Drawing = 3,
    #[assoc(duration = 204)]
    HBlank = 0,
    #[assoc(duration = 456)]
    VBlank = 1,
}

impl Mode {
    pub fn is_oam_scam(&self) -> bool {
        matches!(self, Self::OAMScan)
    }
    pub fn is_drawing(&self) -> bool {
        matches!(self, Self::Drawing)
    }
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
    priority: bool,
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
        let stat: Stat = Default::default();
        Self {
            video_buffer: Default::default(),
            mode_timer: stat.ppu_mode().duration().into(),
            vram: vec![0; (VRAM_BANK_SIZE as usize) * 2].into_boxed_slice(),
            vram_bank: Default::default(),
            oam: vec![0; OAM_SIZE as usize].into_boxed_slice(),
            oam_lock: false,
            bcg_palette_mem: [0; PALLETE_MEM_SIZE],
            obj_palette_mem: [0; PALLETE_MEM_SIZE],
            dmg_palette: None,
            lcdc: Default::default(),
            stat,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            opri: false,
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            bgpi: 0,
            ocpi: 0,
        }
    }
}

pub trait PpuContext {
    fn signal_vblank_interrupt(&mut self);
    fn signal_lcd_interrupt(&mut self);
    fn is_double_speed(&self) -> bool;
    fn is_dmg_compatible(&self) -> bool;
}

impl Ppu {
    pub fn cycle(&mut self, ctx: &mut impl PpuContext) {
        if !self.lcdc.ppu_enable() {
            return;
        }
        self.mode_timer -= if ctx.is_double_speed() { 2 } else { 4 };
        if self.mode_timer > 0 {
            return;
        }
        match self.stat.ppu_mode() {
            Mode::OAMScan => self.switch_mode(Mode::Drawing),
            Mode::Drawing => {
                self.switch_mode(Mode::HBlank);
                self.draw_line(ctx);
            }
            Mode::HBlank => {
                self.ly += 1;
                if self.ly == 144 {
                    ctx.signal_vblank_interrupt();
                    self.switch_mode(Mode::VBlank);
                } else {
                    self.switch_mode(Mode::OAMScan);
                }
            }
            Mode::VBlank => {
                self.ly += 1;
                if self.ly == 154 {
                    self.ly = 0;
                    self.switch_mode(Mode::OAMScan);
                } else {
                    self.mode_timer += Mode::VBlank.duration() as i32;
                }
            }
        };
    }
    pub fn signal_cycle_end(&mut self) {
        self.oam_lock = false;
    }
    /// Read VRAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn cycle_read_vram(&mut self, ctx: &mut impl PpuContext, addr: u16) -> u8 {
        let mode = self.stat.ppu_mode();
        self.cycle(ctx);
        if mode.is_drawing() {
            ().into_data()
        } else {
            self.read_vram(addr)
        }
    }
    /// Write VRAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Writes nothing if the PPU is in the wrong mode
    pub fn cycle_write_vram(&mut self, ctx: &mut impl PpuContext, addr: u16, data: u8) {
        let mode = self.stat.ppu_mode();
        if !mode.is_drawing() {
            self.write_vram(addr, data);
        }
        self.cycle(ctx);
    }
    /// Read OAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn cycle_read_oam(&mut self, ctx: &mut impl PpuContext, addr: u16) -> u8 {
        let mode = self.stat.ppu_mode();
        self.cycle(ctx);
        if mode.is_oam_scam() || mode.is_drawing() || self.oam_lock {
            ().into_data()
        } else {
            self.read_oam(addr)
        }
    }
    /// Write OAM and cycle the PPU. Address is expected to be relative to global gb memory
    /// Writes nothing if the PPU is in the wrong mode
    pub fn cycle_write_oam(&mut self, ctx: &mut impl PpuContext, addr: u16, data: u8) {
        let mode = self.stat.ppu_mode();
        if !mode.is_oam_scam() && !mode.is_drawing() && !self.oam_lock {
            self.write_oam(addr, data);
        }
        self.cycle(ctx);
    }
    pub fn cycle_read(&mut self, ctx: &mut impl PpuContext, f: impl FnOnce(&Self) -> u8) -> u8 {
        self.cycle(ctx);
        f(self)
    }
    pub fn cycle_write(&mut self, ctx: &mut impl PpuContext, f: impl FnOnce(&mut Self)) {
        f(self);
        self.cycle(ctx);
    }
    /// Read VRAM. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[self.to_vram_address(addr) as usize]
    }
    /// Write VRAM. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn write_vram(&mut self, addr: u16, data: u8) {
        self.vram[self.to_vram_address(addr) as usize] = data
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
    /// Read OAM. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn read_oam(&self, addr: u16) -> u8 {
        self.oam[self.to_oam_address(addr) as usize]
    }
    /// Write OAM. Address is expected to be relative to global gb memory
    /// Will read garbage if the PPU is in the wrong mode
    pub fn write_oam(&mut self, addr: u16, data: u8) {
        self.oam[self.to_oam_address(addr) as usize] = data
    }
    fn to_oam_address(&self, addr: u16) -> u16 {
        debug_assert!(
            matches!(addr, OAM_START_ADDR..OAM_END_ADDR),
            "Invalid address for OAM: {addr:04x}"
        );
        addr - OAM_START_ADDR
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
    pub fn read_ly(&self) -> u8 {
        self.ly
    }
    pub fn read_bgpd(&self) -> u8 {
        if self.stat.ppu_mode().is_drawing() {
            ().into_data()
        } else {
            self.bcg_palette_mem[(self.bgpi & PALETTE_MEM_BITS) as usize]
        }
    }
    pub fn write_bgpd(&mut self, data: u8) {
        if !self.stat.ppu_mode().is_drawing() {
            self.bcg_palette_mem[(self.bgpi & PALETTE_MEM_BITS) as usize] = data
        }
        // Autoincrement
        if self.bgpi & 0x80 != 0 {
            self.bgpi += 1
        }
    }
    pub fn read_ocpd(&self) -> u8 {
        if self.stat.ppu_mode().is_drawing() {
            ().into_data()
        } else {
            self.obj_palette_mem[(self.ocpi & PALETTE_MEM_BITS) as usize]
        }
    }
    pub fn write_ocpd(&mut self, data: u8) {
        if !self.stat.ppu_mode().is_drawing() {
            self.obj_palette_mem[(self.ocpi & PALETTE_MEM_BITS) as usize] = data
        }
        // Autoincrement
        if self.ocpi & 0x80 != 0 {
            self.ocpi += 1
        }
    }
    pub fn get_bcg_palettes(&self) -> [Palette; 8] {
        Palette::from_bytes(self.bcg_palette_mem)
    }
    pub fn get_obj_palettes(&self) -> [Palette; 8] {
        Palette::from_bytes(self.obj_palette_mem)
    }
    pub fn write_bgp(&mut self, mut data: u8) {
        self.bgp = data;
        let mut bg_palette = Palette::from_bytes(self.bcg_palette_mem);
        for i in 0..4 {
            let id = data & 0b11;
            data >>= 2;
            let dmg_palette = self.dmg_palette.unwrap_or(Palette::from_colors([
                PaletteColor::white(),
                PaletteColor::light_grey(),
                PaletteColor::dark_grey(),
                PaletteColor::black(),
            ]));
            bg_palette[0].colors[i] = dmg_palette.colors[id as usize];
        }
        self.bcg_palette_mem = Palette::to_bytes(bg_palette);
    }
    pub fn read_vbk(&self) -> u8 {
        self.vram_bank as u8
    }
    pub fn write_vbk(&mut self, data: u8) {
        self.vram_bank = data & 1 != 0
    }
    pub fn set_dmg_palette(&mut self) {
        let pals = self.get_bcg_palettes();
        self.dmg_palette = Some(pals[0]);
    }
    fn switch_mode(&mut self, mode: Mode) {
        self.stat.set_ppu_mode(mode);
        self.mode_timer += mode.duration() as i32;
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
        &self.video_buffer
    }
    fn draw_line(&mut self, ctx: &mut impl PpuContext) {
        self.video_buffer
            .init_line_pass(self.ly, self.bcg_palette_mem, self.obj_palette_mem);
        for px in &mut self.video_buffer.lines[self.ly as usize].pixels {
            px.set_lcdc_bg_enable_or_priority(self.lcdc.bg_enable_or_priority());
        }
        if self.lcdc.obj_enable() {
            let objects = self.get_objects_in_ly();

            let ly = self.ly as i32;
            let obj_height = if self.lcdc.obj_size() { 16 } else { 8 };
            for obj in objects {
                let obj_screen_y = obj.y as i32 - 16;
                let obj_screen_x = obj.x as i32 - 8;
                let obj_sprite_y = if obj.attrs.y_flip() {
                    obj_screen_y + obj_height - 1 - ly
                } else {
                    ly - obj_screen_y
                };
                debug_assert!(
                    obj_sprite_y >= 0 && obj_sprite_y < obj_height,
                    "get_objects_in_ly should only return valid objects for the current line",
                );
                let is_second_tile = obj_sprite_y > 7;
                let tile_index = obj.tile_index.wrapping_add(is_second_tile as u8);
                let tile = self.get_object_tile_bytes(tile_index, obj.attrs);
                let tile_y = obj_sprite_y as usize % 8;

                let mut lo_byte = tile[2 * tile_y];
                let mut hi_byte = tile[2 * tile_y + 1];
                // bits are normally flipped from the order we visit them
                if !obj.attrs.x_flip() {
                    lo_byte = lo_byte.reverse_bits();
                    hi_byte = hi_byte.reverse_bits();
                }
                for x in obj_screen_x..(obj_screen_x + 8) {
                    if !(0..WINDOW_WIDTH as i32).contains(&x) {
                        continue;
                    }
                    let lx = x as u8;
                    let color_index = ((hi_byte & 1) << 1) | (lo_byte & 1);

                    self.video_buffer.set_object_pixel(
                        lx,
                        self.ly,
                        obj.attrs.cgb_palette(),
                        color_index,
                        obj.attrs.priority(),
                    );

                    lo_byte >>= 1;
                    hi_byte >>= 1;
                }
            }
        }
        if !ctx.is_dmg_compatible() || self.lcdc.bg_enable_or_priority() {
            let y = self.ly.wrapping_add(self.scy);
            let map_y = (y / 8) as usize;
            let map_start: usize = if self.lcdc.bg_tile_map_area() {
                0x1C00
            } else {
                0x1800
            };

            let mut lx = 0;
            while lx < WINDOW_WIDTH {
                let x = lx.wrapping_add(self.scx);
                let map_x = (x / 8) as usize;

                let timemap_index = map_start + map_y * 32 + map_x;
                let tile_attrs = BackgroundAttributes::from(self.vram[timemap_index + 0x2000]);
                let tile_index = self.vram[timemap_index];
                let tile = self.get_background_tile_bytes(tile_index, tile_attrs);
                let tile_y = if tile_attrs.y_flip() {
                    7 - (y % 8) as usize
                } else {
                    (y % 8) as usize
                };
                let mut lo_byte = tile[tile_y * 2];
                let mut hi_byte = tile[tile_y * 2 + 1];
                // bits are normally flipped from the order we visit them
                if !tile_attrs.x_flip() {
                    lo_byte = lo_byte.reverse_bits();
                    hi_byte = hi_byte.reverse_bits();
                }
                // skip the first few pixels to the left of the screen
                let tile_quot = x % 8;
                lo_byte >>= tile_quot;
                hi_byte >>= tile_quot;

                loop {
                    let color_index = ((hi_byte & 1) << 1) | (lo_byte & 1);

                    self.video_buffer.set_background_pixel(
                        lx,
                        self.ly,
                        tile_attrs.color_palette(),
                        color_index,
                        tile_attrs.priority(),
                    );

                    lx += 1;
                    let new_map_x = (lx.wrapping_add(self.scx) / 8) as usize;
                    if map_x != new_map_x || lx == WINDOW_WIDTH {
                        break;
                    }
                    lo_byte >>= 1;
                    hi_byte >>= 1;
                }
            }
        }
    }
    /// returns the (up to) 10 objects that can be drawn in the current line
    /// sorted by priority in decreasing order
    fn get_objects_in_ly(&self) -> ArrayVec<Object, 10> {
        let obj_height = if self.lcdc.obj_size() { 16 } else { 8 };
        let mut objects: ArrayVec<(usize, Object), 10> = self
            .oam
            .chunks_exact(4)
            .filter_map(|bytes| {
                let object = Object::from_bytes(bytes);
                let y0 = object.y as i32 - 16;
                let y1 = y0 + obj_height;
                if (y0..y1).contains(&(self.ly as i32)) {
                    Some(object)
                } else {
                    None
                }
            })
            .take(10)
            .enumerate()
            .collect();
        // in CGB mode, the earlier the obj the higher the priority
        // if opri is set, DMG priority is used instead
        if self.opri {
            // in DMG mode, the smaller the obj x, the higher the priority
            objects.sort_by(|&(a_index, a), &(b_index, b)| match a.x.cmp(&b.x) {
                // untie by found order
                std::cmp::Ordering::Equal => a_index.cmp(&b_index),
                not_equal => not_equal.reverse(),
            });
        }
        objects.into_iter().map(|(_, obj)| obj).collect()
    }
    fn get_background_tile_bytes(&self, tile_index: u8, attrs: BackgroundAttributes) -> &[u8] {
        let tile_index = if self.lcdc.bg_tile_addr_mode() {
            tile_index
        } else {
            (tile_index as i8 as i16 + 256) as u8
        };
        self.get_tile_bytes(tile_index, attrs.bank())
    }
    fn get_object_tile_bytes(&self, tile_index: u8, attrs: ObjectAttributes) -> &[u8] {
        self.get_tile_bytes(tile_index, attrs.bank())
    }
    fn get_tile_bytes(&self, tile_index: u8, bank: u8) -> &[u8] {
        let tile_index = tile_index as usize;
        let bank = bank as usize;
        let start = tile_index * 16 + bank * 0x2000;
        let end = start + 16;
        &self.vram[start..end]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Object {
    x: u8,
    y: u8,
    tile_index: u8,
    attrs: ObjectAttributes,
}

impl Object {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            x: bytes[1],
            y: bytes[0],
            tile_index: bytes[2],
            attrs: ObjectAttributes::from_bytes([bytes[3]]),
        }
    }
}

#[bitfield(bits = 16)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct PaletteColor {
    red: B5,
    green: B5,
    blue: B5,
    #[skip]
    __: B1,
}

impl PaletteColor {
    pub fn white() -> Self {
        Self::new().with_red(0x1F).with_green(0x1F).with_blue(0x1F)
    }
    pub fn light_grey() -> Self {
        Self::new().with_red(0x16).with_green(0x16).with_blue(0x16)
    }
    pub fn dark_grey() -> Self {
        Self::new().with_red(0x0C).with_green(0x0B).with_blue(0x0B)
    }
    pub fn black() -> Self {
        Self::new().with_red(0x00).with_green(0x00).with_blue(0x00)
    }
    pub fn rgb(self) -> RgbColor {
        self.into()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    pub fn red(&self) -> u8 {
        self.red
    }
    pub fn green(&self) -> u8 {
        self.green
    }
    pub fn blue(&self) -> u8 {
        self.blue
    }
}

impl From<PaletteColor> for RgbColor {
    fn from(value: PaletteColor) -> Self {
        Self {
            red: value.red() << 3,
            green: value.green() << 3,
            blue: value.blue() << 3,
        }
    }
}

impl From<RgbColor> for PaletteColor {
    fn from(value: RgbColor) -> Self {
        Self::new()
            .with_red(value.red >> 3)
            .with_green(value.green >> 3)
            .with_blue(value.blue >> 3)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Palette {
    colors: [PaletteColor; Self::COLORS_PER_PALETTE],
}

impl Palette {
    pub const NUM_PALETTES: usize = 8;
    pub const COLORS_PER_PALETTE: usize = 4;
    pub const BYTES_PER_COLOR: usize = 2;

    pub fn from_bytes(bytes: [u8; PALLETE_MEM_SIZE]) -> [Self; Self::NUM_PALETTES] {
        // SAFETY: any memory state is a valid palette
        unsafe { std::mem::transmute(bytes) }
    }
    pub fn to_bytes(slf: [Self; Self::NUM_PALETTES]) -> [u8; PALLETE_MEM_SIZE] {
        unsafe { std::mem::transmute(slf) }
    }
    fn from_colors(colors: [PaletteColor; Self::COLORS_PER_PALETTE]) -> Self {
        Self { colors }
    }
    pub fn get_rgb_color(&self, index: usize) -> RgbColor {
        self.colors[index].into()
    }
}

pub struct VideoBuffer {
    lines: [VideoLine; WINDOW_HEIGHT as usize],
}

impl Default for VideoBuffer {
    fn default() -> Self {
        Self {
            lines: core::array::from_fn(|_| Default::default()),
        }
    }
}

struct VideoLine {
    background_palettes: [Palette; 8],
    object_palettes: [Palette; 8],
    pixels: [PixelInfo; WINDOW_WIDTH as usize],
}

impl Default for VideoLine {
    fn default() -> Self {
        Self {
            background_palettes: Default::default(),
            object_palettes: Default::default(),
            pixels: core::array::from_fn(|_| Default::default()),
        }
    }
}

impl VideoLine {
    fn get_background_color(&self, frag: FragmentInfo) -> PaletteColor {
        self.background_palettes[frag.palette_index() as usize].colors[frag.color_index() as usize]
    }
    fn get_object_color(&self, frag: FragmentInfo) -> PaletteColor {
        self.object_palettes[frag.palette_index() as usize].colors[frag.color_index() as usize]
    }
}

#[bitfield(bits = 5)]
#[derive(Debug, Default, Specifier)]
struct FragmentInfo {
    palette_index: B3,
    color_index: B2,
}

#[bitfield(bytes = 2)]
#[derive(Debug, Default)]
struct PixelInfo {
    background_fragment: FragmentInfo,
    has_background: bool,
    object_fragment: FragmentInfo,
    has_object: bool,
    lcdc_bg_enable_or_priority: bool,
    oam_obj_priority: bool,
    bg_attr_priority: bool,
    #[skip]
    __: B1,
}

impl VideoBuffer {
    pub fn make_rgba_buffer(&self) -> Box<[u8]> {
        puffin::profile_function!();
        let mut buffer = Vec::with_capacity(WINDOW_BUFFER_SIZE * 4);
        for line in &self.lines {
            for px in &line.pixels {
                let color: PaletteColor;
                match (px.has_background(), px.has_object()) {
                    (false, false) => {
                        color = PaletteColor::white();
                    }
                    (false, true) => {
                        color = line.get_object_color(px.object_fragment());
                    }
                    (true, false) => {
                        color = line.get_background_color(px.background_fragment());
                    }
                    (true, true) => {
                        let bg_frag = px.background_fragment();
                        if bg_frag.color_index() != 0
                            && px.lcdc_bg_enable_or_priority()
                            && (px.oam_obj_priority() || px.bg_attr_priority())
                        {
                            color = line.get_background_color(bg_frag)
                        } else {
                            color = line.get_object_color(px.object_fragment())
                        }
                    }
                }

                let r = color.red() as u16;
                let g = color.green() as u16;
                let b = color.blue() as u16;
                buffer.push(((r * 13 + g * 2 + b) / 2) as u8);
                buffer.push(((g * 3 + b) * 2) as u8);
                buffer.push(((r * 3 + g * 2 + b * 11) / 2) as u8);
                buffer.push(0xFF);
            }
        }
        buffer.into_boxed_slice()
    }
    fn set_line_background_palettes(&mut self, ly: u8, palettes: [u8; PALLETE_MEM_SIZE]) {
        self.lines[ly as usize].background_palettes = Palette::from_bytes(palettes)
    }
    fn set_line_object_palettes(&mut self, ly: u8, palettes: [u8; PALLETE_MEM_SIZE]) {
        self.lines[ly as usize].object_palettes = Palette::from_bytes(palettes)
    }

    fn set_background_pixel(
        &mut self,
        lx: u8,
        ly: u8,
        palette_index: u8,
        color_index: u8,
        priority: bool,
    ) {
        let px = &mut self.lines[ly as usize].pixels[lx as usize];
        if px.has_background() {
            return;
        }
        px.set_has_background(true);
        px.set_background_fragment(
            FragmentInfo::new()
                .with_palette_index(palette_index)
                .with_color_index(color_index),
        );
        px.set_bg_attr_priority(priority);
    }
    fn set_object_pixel(
        &mut self,
        lx: u8,
        ly: u8,
        palette_index: u8,
        color_index: u8,
        priority: bool,
    ) {
        let px = &mut self.lines[ly as usize].pixels[lx as usize];
        if px.has_object() {
            return;
        }
        px.set_has_object(true);
        px.set_object_fragment(
            FragmentInfo::new()
                .with_palette_index(palette_index)
                .with_color_index(color_index),
        );
        px.set_oam_obj_priority(priority);
    }
    fn has_background(&self, lx: u8, ly: u8) -> bool {
        self.lines[ly as usize].pixels[lx as usize].has_background()
    }
    fn has_object(&self, lx: u8, ly: u8) -> bool {
        self.lines[ly as usize].pixels[lx as usize].has_object()
    }
    fn init_line_pass(
        &mut self,
        ly: u8,
        background_palettes: [u8; PALLETE_MEM_SIZE],
        object_palettes: [u8; PALLETE_MEM_SIZE],
    ) {
        self.set_line_background_palettes(ly, background_palettes);
        self.set_line_object_palettes(ly, object_palettes);
        for px_info in &mut self.lines[ly as usize].pixels {
            px_info.set_has_background(false);
            px_info.set_has_object(false);
        }
    }
}

use arrayvec::ArrayVec;
use enum_assoc::Assoc;
use modular_bitfield::prelude::*;
use std::{
    cell::{Cell, Ref, RefCell},
    iter::Zip,
    rc::{Rc, Weak},
};

use super::{
    Addressable, Bus, OUTPUT_HEIGHT, OUTPUT_WIDTH,
    bus::AddressingError,
    constants::{INTERRUPT_LCD_BIT, INTERRUPTS_ADDR},
    variant::GameBoyVariant,
};

#[derive(Debug, Clone, Copy, Assoc)]
#[func(pub fn duration(&self) -> u16)]
pub enum Mode {
    #[assoc(duration = 80)]
    OAMScan = 2,
    #[assoc(duration = 240)]
    Drawing = 3,
    #[assoc(duration = 136)]
    HBlank = 0,
    #[assoc(duration = 456)]
    VBlank = 1,
}

pub struct PPUData {
    frame_buffer: Box<[Pixel]>,
    frame_ready: bool,
    variant: Rc<GameBoyVariant>,
    vram: Box<[u8]>,
    oam: Box<[u8]>,
    bcg_palette_mem: [u8; 64],
    obj_palette_mem: [u8; 64],
    pub lcdc: Lcdc,
    delay: u16,
    pub mode: Mode,
    scx: u8,
    scy: u8,
    wx: u8,
    wy: u8,
    window_line: u8,
    pub ly: u8,
    lyc: u8,
    pub stat: Stat,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    bcps: u8,
    ocps: u8,
}

impl PPUData {
    fn new(variant: Rc<GameBoyVariant>, vram: Box<[u8]>, frame_buffer: Box<[Pixel]>) -> Self {
        Self {
            frame_buffer,
            frame_ready: false,
            variant,
            vram,
            oam: vec![0; 0xA0].into_boxed_slice(),
            bcg_palette_mem: [0; 64],
            obj_palette_mem: [0; 64],
            lcdc: Lcdc::new(),
            delay: 0,
            mode: Mode::OAMScan,
            scx: 0,
            scy: 0,
            wx: 0,
            wy: 0,
            window_line: 0,
            ly: 0,
            lyc: 0,
            stat: Stat::new(),
            bgp: 0,
            obp0: 0,
            obp1: 0,
            bcps: 0,
            ocps: 0,
        }
    }
    fn bcps_autoincrement(&self) -> bool {
        self.bcps >> 7 != 0
    }
    fn bcps_address(&self) -> u8 {
        self.bcps & 0b11111
    }
    fn ocps_autoincrement(&self) -> bool {
        self.ocps >> 7 != 0
    }
    fn ocps_address(&self) -> u8 {
        self.ocps & 0b11111
    }
    fn ppu_mode(&self) -> u8 {
        if !self.lcdc.ppu_enable() {
            0
        } else {
            self.mode as u8
        }
    }
    fn lyc_eq_ly(&self) -> bool {
        self.lyc == self.ly
    }
    fn get_stat(&self) -> Stat {
        let mut stat = self.stat;
        stat.set_ppu_mode(self.ppu_mode());
        stat.set_lyc_eq_ly(self.lyc_eq_ly());
        stat
    }
    fn stat_interrupt_line(&self) -> bool {
        let mut res = false;
        let stat = self.get_stat();
        if stat.lyc_int_select() {
            res |= stat.lyc_eq_ly();
        }
        if stat.mode0_int_select() {
            res |= (self.mode as u8) == 0
        }
        if stat.mode1_int_select() {
            res |= (self.mode as u8) == 1
        }
        if stat.mode2_int_select() {
            res |= (self.mode as u8) == 2
        }
        res
    }
    fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.delay = mode.duration();
    }
    fn clock(&mut self) {
        if !self.lcdc.ppu_enable() {
            return;
        }
        if self.delay > 1 {
            self.delay -= 1;
            return;
        }
        match self.mode {
            Mode::OAMScan => self.switch_mode(Mode::Drawing),
            Mode::Drawing => {
                self.draw_scanline();
                self.switch_mode(Mode::HBlank);
            }
            Mode::HBlank => {
                self.ly += 1;
                if self.ly == 144 {
                    self.frame_ready = true;
                    self.switch_mode(Mode::VBlank);
                } else {
                    self.switch_mode(Mode::OAMScan);
                }
            }
            Mode::VBlank => {
                self.ly += 1;
                self.window_line = 0;
                if self.ly == 154 {
                    self.ly = 0;
                    self.frame_ready = false;
                    self.switch_mode(Mode::OAMScan);
                } else {
                    self.delay += Mode::VBlank.duration();
                }
            }
        }
        self.stat.set_lyc_eq_ly(self.ly == self.lyc);
    }
    fn draw_scanline(&mut self) {
        // here comes the pain

        let mut obj_pixel_info: [Option<(u8, ObjectAttributes)>; OUTPUT_WIDTH as usize] =
            [None; OUTPUT_WIDTH as usize];
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
                let is_second_tile = obj_sprite_y > 7;
                let tile = self.get_tile_bytes(
                    obj.tile_index.wrapping_add(is_second_tile as u8) as usize,
                    if !self.variant.is_dmg_compatible() {
                        obj.attrs.bank()
                    } else {
                        0
                    },
                );
                let tile_y = obj_sprite_y as usize % 8;
                let mut lo_byte = tile[2 * tile_y];
                let mut hi_byte = tile[2 * tile_y + 1];

                if !obj.attrs.x_flip() {
                    lo_byte = lo_byte.reverse_bits();
                    hi_byte = hi_byte.reverse_bits();
                }
                for x in (obj_screen_x)..(obj_screen_x + 8) {
                    if !(0..OUTPUT_WIDTH as i32).contains(&x) {
                        continue;
                    }
                    let palette_index = (hi_byte & 1) + (lo_byte & 1);
                    if palette_index != 0 {
                        obj_pixel_info[x as usize] = Some((palette_index, obj.attrs));
                    }
                    lo_byte >>= 1;
                    hi_byte >>= 1;
                }
            }
        }
        let mut bg_pixel_info: [(u8, Option<BackgroundAttributes>); OUTPUT_WIDTH as usize] =
            [(0, None); OUTPUT_WIDTH as usize];
        if !self.variant.is_dmg_compatible() || self.lcdc.bg_enable_or_priority() {
            let y = self.ly.wrapping_add(self.scy);
            let map_y = (y / 8) as usize;
            let map_start: usize = if self.lcdc.bg_tile_map_area() {
                0x1C00
            } else {
                0x1800
            };

            let mut lx = 0;
            while lx < OUTPUT_WIDTH {
                let x = lx.wrapping_add(self.scx);
                let map_x = (x / 8) as usize;

                let tile_map_index = map_start + map_y * 32 + map_x;
                let tile_attrs_opt = if !self.variant.is_dmg_compatible() {
                    Some(BackgroundAttributes::from_bytes([
                        self.vram[tile_map_index + 0x2000]
                    ]))
                } else {
                    None
                };
                let tile_index = self.vram[tile_map_index] as usize;
                let tile_index = if self.lcdc.bg_tile_addr_mode() {
                    tile_index
                } else {
                    (tile_index as i8 as i16 + 256) as usize
                };
                let tile = self.get_tile_bytes(
                    tile_index,
                    tile_attrs_opt.map(|attrs| attrs.bank()).unwrap_or(0),
                );
                let tile_y = if tile_attrs_opt.map(|attrs| attrs.y_flip()).unwrap_or(false) {
                    (y % 8).wrapping_sub(7).wrapping_mul(0xFF) as usize
                } else {
                    (y % 8) as usize
                };
                let mut lo_byte = tile[2 * tile_y];
                let mut hi_byte = tile[2 * tile_y + 1];
                // bits are normally flipped from the order we visit them
                if !tile_attrs_opt.map(|attrs| attrs.x_flip()).unwrap_or(false) {
                    lo_byte = lo_byte.reverse_bits();
                    hi_byte = hi_byte.reverse_bits();
                }
                // skip the first few pixels to the left of the screen
                let tile_quot = x % 8;
                lo_byte >>= tile_quot;
                hi_byte >>= tile_quot;

                loop {
                    let palette_index = (hi_byte & 1) + (lo_byte & 1);

                    bg_pixel_info[lx as usize] = (palette_index, tile_attrs_opt);

                    lx += 1;
                    let new_map_x = (lx.wrapping_add(self.scx) / 8) as usize;
                    if map_x != new_map_x || lx == OUTPUT_WIDTH {
                        break;
                    }
                    lo_byte >>= 1;
                    hi_byte >>= 1;
                }
            }
        }
        let buffer_start = self.ly as usize * OUTPUT_WIDTH as usize;
        for (i, (obj, (bg_palette_index, bg_attrs_opt))) in obj_pixel_info
            .into_iter()
            .zip(bg_pixel_info.into_iter())
            .enumerate()
        {
            let obj_priority = obj
                .map(|(_, obj_attrs)| {
                    bg_palette_index == 0
                        || !self.lcdc.bg_enable_or_priority()
                        || (!obj_attrs.not_priority()
                            && !bg_attrs_opt.map(|attrs| attrs.priority()).unwrap_or(false))
                })
                .unwrap_or(false);
            self.frame_buffer[buffer_start + i] = if obj_priority {
                let Some((obj_palette_index, obj_attrs)) = obj else {
                    unreachable!()
                };
                self.get_obj_palette_color(obj_palette_index, obj_attrs)
            } else if self.variant.is_dmg_compatible() && !self.lcdc.bg_enable_or_priority() {
                Pixel::white()
            } else {
                self.get_bg_palette_color(bg_palette_index, bg_attrs_opt)
            };
        }
    }
    /// returns the (up to) 10 objects that can be drawn in the current line
    /// sorted by priority in increasing order
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
        if self.variant.is_dmg_compatible() {
            // in DMG mode, the smaller the obj x, the higher the priority
            objects.sort_by(|&(a_index, a), &(b_index, b)| match a.x.cmp(&b.x) {
                // untie by reverse found order
                std::cmp::Ordering::Equal => a_index.cmp(&b_index).reverse(),
                not_equal => not_equal.reverse(),
            });
        } else {
            // in CGB mode, the earlier the obj the higher the priority
            // which is just reverse found order
            objects.reverse();
        }
        objects.into_iter().map(|(_, obj)| obj).collect()
    }
    fn get_tile_bytes(&self, tile_index: usize, bank: u8) -> &[u8] {
        let start = tile_index * 16 + bank as usize * 0x2000;
        let end = start + 16;
        &self.vram[start..end]
    }
    fn get_obj_palette_color(&self, palette_index: u8, attrs: ObjectAttributes) -> Pixel {
        if self.variant.is_color_variant() && self.variant.is_dmg_compatible() {
            todo!()
        } else if self.variant.is_color_variant() {
            todo!()
        } else {
            let mut color_register = if attrs.dmg_palette() == 0 {
                self.obp0
            } else {
                self.obp1
            };
            for _ in 0..palette_index {
                color_register >>= 2;
            }
            color_register &= 0b11;
            match color_register {
                0 => Pixel::white(),
                1 => Pixel::light_gray(),
                2 => Pixel::dark_gray(),
                3 => Pixel::black(),
                _ => unreachable!(),
            }
        }
    }
    fn get_bg_palette_color(
        &self,
        palette_index: u8,
        attrs: Option<BackgroundAttributes>,
    ) -> Pixel {
        if self.variant.is_color_variant() && self.variant.is_dmg_compatible() {
            todo!()
        } else if self.variant.is_color_variant() {
            todo!()
        } else {
            let mut color_register = self.bgp;
            for _ in 0..palette_index {
                color_register >>= 2;
            }
            color_register &= 0b11;
            match color_register {
                0 => Pixel::white(),
                1 => Pixel::light_gray(),
                2 => Pixel::dark_gray(),
                3 => Pixel::black(),
                _ => unreachable!(),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Pixel {
    pub fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
    pub fn white() -> Self {
        Self {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }
    }
    pub fn light_gray() -> Self {
        Self {
            r: 0x60,
            g: 0x60,
            b: 0x60,
        }
    }
    pub fn dark_gray() -> Self {
        Self {
            r: 0xB0,
            g: 0xB0,
            b: 0xB0,
        }
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

#[bitfield(bits = 8)]
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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
pub struct Stat {
    ppu_mode: B2,
    lyc_eq_ly: bool,
    mode0_int_select: bool,
    mode1_int_select: bool,
    mode2_int_select: bool,
    lyc_int_select: bool,
    #[skip]
    __: B1,
}

pub struct Ppu {
    variant: Rc<GameBoyVariant>,
    bus: Rc<Bus>,
    data: RefCell<PPUData>,
}

impl Ppu {
    pub fn new(variant: Rc<GameBoyVariant>, bus: Rc<Bus>) -> Self {
        let vram_size = if variant.is_color_variant() {
            16 << 10
        } else {
            8 << 10
        };
        let vram = vec![0u8; vram_size].into_boxed_slice();
        let data = PPUData::new(
            Rc::clone(&variant),
            vram,
            vec![Pixel::black(); OUTPUT_WIDTH as usize * OUTPUT_HEIGHT as usize].into_boxed_slice(),
        );
        Self {
            variant,
            bus,
            data: RefCell::new(data),
        }
    }
    pub fn data(&self) -> Ref<PPUData> {
        self.data.borrow()
    }
    pub fn frame_is_ready(&self) -> bool {
        self.data.borrow().frame_ready
    }
    pub fn get_frame(&self) -> Box<[Pixel]> {
        let mut data = self.data.borrow_mut();
        data.frame_ready = false;
        data.frame_buffer.clone()
    }
    pub fn clock(&self) {
        let mut data = self.data.borrow_mut();
        let stat_itr_line0 = data.stat_interrupt_line();
        data.clock();
        let stat_itr_line1 = data.stat_interrupt_line();
        let rising_edge = !stat_itr_line0 && stat_itr_line1;
        if rising_edge {
            let ints = self.bus.read(INTERRUPTS_ADDR);
            self.bus.write(INTERRUPTS_ADDR, ints | INTERRUPT_LCD_BIT);
        }
    }
}

impl Addressable for Ppu {
    fn size(&self) -> usize {
        0xFF
    }

    fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
        match addr {
            0x40 => Ok(self.data.borrow().lcdc.bytes[0]),
            0x41 => Ok(self.data.borrow().get_stat().bytes[0]),
            0x42 => Ok(self.data.borrow().scx),
            0x43 => Ok(self.data.borrow().scy),
            0x44 => Ok(self.data.borrow().ly),
            0x45 => Ok(self.data.borrow().lyc),
            0x47 if self.variant.is_dmg_compatible() => Ok(self.data.borrow().bgp),
            0x48 if self.variant.is_dmg_compatible() => Ok(self.data.borrow().obp0),
            0x49 if self.variant.is_dmg_compatible() => Ok(self.data.borrow().obp1),
            0x4A => Ok(self.data.borrow().wy),
            0x4B => Ok(self.data.borrow().wx),
            0x68 if !self.variant.is_dmg_compatible() => Ok(self.data.borrow().bcps),
            0x69 if !self.variant.is_dmg_compatible() => {
                let data = self.data.borrow();
                Ok(data.bcg_palette_mem[data.bcps_address() as usize])
            }
            0x6A if !self.variant.is_dmg_compatible() => Ok(self.data.borrow().ocps),
            0x6B if !self.variant.is_dmg_compatible() => {
                let data = self.data.borrow();
                Ok(data.obj_palette_mem[data.ocps_address() as usize])
            }
            _ => Err(AddressingError::Unmapped),
        }
    }

    fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError> {
        match addr {
            0x40 => {
                self.data.borrow_mut().lcdc = Lcdc::from_bytes([data]);
                Ok(())
            }
            0x41 => {
                self.data.borrow_mut().stat = Stat::from_bytes([data]);
                Ok(())
            }
            0x42 => {
                self.data.borrow_mut().scx = data;
                Ok(())
            }
            0x43 => {
                self.data.borrow_mut().scy = data;
                Ok(())
            }
            0x45 => {
                self.data.borrow_mut().lyc = data;
                Ok(())
            }
            0x47 if self.variant.is_dmg_compatible() => {
                self.data.borrow_mut().bgp = data;
                Ok(())
            }
            0x48 if self.variant.is_dmg_compatible() => {
                self.data.borrow_mut().obp0 = data;
                Ok(())
            }
            0x49 if self.variant.is_dmg_compatible() => {
                self.data.borrow_mut().obp1 = data;
                Ok(())
            }
            0x4A => {
                self.data.borrow_mut().wy = data;
                Ok(())
            }
            0x4B => {
                self.data.borrow_mut().wx = data;
                Ok(())
            }
            0x68 if !self.variant.is_dmg_compatible() => {
                self.data.borrow_mut().bcps = data;
                Ok(())
            }
            0x69 if !self.variant.is_dmg_compatible() => {
                let mut ppu_data = self.data.borrow_mut();
                let addr = ppu_data.bcps_address() as usize;
                ppu_data.bcg_palette_mem[addr] = data;
                if ppu_data.bcps_autoincrement() {
                    if addr == 0b11111 {
                        ppu_data.bcps &= 0b11100000;
                    } else {
                        ppu_data.bcps += 1;
                    }
                }
                Ok(())
            }
            0x6A if !self.variant.is_dmg_compatible() => {
                self.data.borrow_mut().ocps = data;
                Ok(())
            }
            0x6B if !self.variant.is_dmg_compatible() => {
                let mut ppu_data = self.data.borrow_mut();
                let addr = ppu_data.ocps_address() as usize;
                ppu_data.obj_palette_mem[addr] = data;
                if ppu_data.ocps_autoincrement() {
                    if addr == 0b11111 {
                        ppu_data.ocps &= 0b11100000;
                    } else {
                        ppu_data.ocps += 1;
                    }
                }
                Ok(())
            }
            _ => Err(AddressingError::Unmapped),
        }
    }
}

pub struct VRam {
    ppu: Rc<Ppu>,
    bank: Cell<u8>,
}

impl VRam {
    pub fn new(ppu: Rc<Ppu>) -> Self {
        Self {
            ppu,
            bank: Cell::new(0),
        }
    }
    fn addr_bank(&self, addr: u16) -> u16 {
        addr + (0x2000 * self.bank.get() as u16)
    }
}

impl Addressable for VRam {
    fn size(&self) -> usize {
        0x2000
    }

    fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
        let vram = &self.ppu.data.borrow().vram;
        Ok(vram[self.addr_bank(addr) as usize])
    }

    fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError> {
        let vram = &mut self.ppu.data.borrow_mut().vram;
        vram[self.addr_bank(addr) as usize] = data;
        Ok(())
    }
}

pub struct VRamBank {
    vram: Rc<VRam>,
}

impl VRamBank {
    pub fn new(vram: Rc<VRam>) -> Self {
        Self { vram }
    }
}

impl Addressable for VRamBank {
    fn size(&self) -> usize {
        1
    }

    fn const_read(&self, _addr: u16) -> Result<u8, AddressingError> {
        Ok(0xFE | self.vram.bank.get())
    }

    fn write(&self, _addr: u16, data: u8) -> Result<(), AddressingError> {
        self.vram.bank.set(data & 1);
        Ok(())
    }
}
pub struct Oam {
    ppu: Rc<Ppu>,
}

impl Oam {
    pub fn new(ppu: Rc<Ppu>) -> Self {
        Self { ppu }
    }
}

impl Addressable for Oam {
    fn size(&self) -> usize {
        0x2000
    }

    fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
        let oam = &self.ppu.data.borrow().oam;
        Ok(oam[addr as usize])
    }

    fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError> {
        let oam = &mut self.ppu.data.borrow_mut().oam;
        oam[addr as usize] = data;
        Ok(())
    }
}

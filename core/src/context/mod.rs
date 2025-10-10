use apu::Apu;
pub use apu::AudioOutput;
use hdma::Hdma;
use hram::HighRam;
use interrupts::{Interrupt, InterruptFlags};
use key1::Key1;
use oam_dma::OamDma;
pub use p1::Input;
use p1::P1;
use ppu::{Ppu, PpuContext};
use timer::Timer;
use wram::WorkRam;

use super::{
    BootRom,
    cartridge::Cartridge,
    context::timer::TimerContext,
    cpu::{CPUState, CpuContext},
    events::{Event, Events},
    time::SystemTime,
};

mod apu;
mod hdma;
mod hram;
pub mod interrupts;
mod key1;
mod oam_dma;
mod p1;
mod ppu;
mod timer;
mod wram;

pub use ppu::{Palette, VideoBuffer};

#[derive(Debug)]
pub struct Context {
    time: SystemTime,
    events: Events,
    dmg_compatibility: bool,
    cartridge: Cartridge,
    boot_rom: BootRom,
    ppu: Ppu,
    oam_dma: OamDma,
    hdma: Hdma,
    wram: WorkRam,
    p1: P1,
    timer: Timer,
    apu: Apu,
    key1: Key1,
    hram: HighRam,
    interrupts: InterruptFlags,
    interrupt_enable: InterruptFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryRegion {
    Cartridge,
    VRam,
    WRam,
    Oam,
    Timer,
    OamDma,
    Hdma,
    HRam,
}

impl MemoryRegion {
    pub fn from_addr(addr: u16) -> Option<Self> {
        let slf = match addr {
            0x0000..0x8000 | 0xA000..0xC000 => Self::Cartridge,
            0x8000..0xA000 => Self::VRam,
            0xC000..0xFE00 => Self::WRam,
            0xFE00..0xFEA0 => Self::Oam,
            0xFF04..0xFF08 => Self::Timer,
            0xFF46 => Self::OamDma,
            0xFF51..0xFF56 => Self::Hdma,
            0xFF80..=0xFFFE => Self::HRam,
            _ => return None,
        };
        Some(slf)
    }
}

trait IntoData {
    fn into_data(self) -> u8;
}

impl IntoData for () {
    fn into_data(self) -> u8 {
        0xFF
    }
}
impl IntoData for u8 {
    fn into_data(self) -> u8 {
        self
    }
}
impl IntoData for InterruptFlags {
    fn into_data(self) -> u8 {
        self.into()
    }
}

impl Context {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            time: Default::default(),
            events: Default::default(),
            dmg_compatibility: false,
            cartridge,
            boot_rom: BootRom::new(),
            ppu: Default::default(),
            oam_dma: Default::default(),
            hdma: Default::default(),
            wram: Default::default(),
            p1: Default::default(),
            timer: Default::default(),
            apu: Default::default(),
            key1: Default::default(),
            hram: Default::default(),
            interrupts: Default::default(),
            interrupt_enable: Default::default(),
        }
    }
    pub fn system_time(&self) -> SystemTime {
        self.time
    }
    pub fn fetch_clear_events(&mut self) -> Events {
        std::mem::take(&mut self.events)
    }
    pub fn press_key(&mut self, input: Input) {
        if self.p1.press(input) {
            self.interrupts.set_joypad(true);
        }
    }
    pub fn unpress_key(&mut self, input: Input) {
        if self.p1.unpress(input) {
            self.interrupts.set_joypad(true);
        }
    }
    pub fn get_video_buffer(&self) -> &VideoBuffer {
        self.ppu.get_video_buffer()
    }
    pub fn get_audio_output(&mut self) -> AudioOutput {
        self.apu.get_audio_output()
    }
    fn active_interrupts(&self) -> InterruptFlags {
        (Into::<u8>::into(self.interrupts) & Into::<u8>::into(self.interrupt_enable)).into()
    }
    pub fn is_double_speed(&self) -> bool {
        self.key1.current_speed()
    }
    fn is_double_speed_cycle(&self) -> bool {
        self.is_double_speed() && !self.time.cycles().is_multiple_of(4)
    }
    fn set_dmg_compatibility(&mut self, compat: bool) {
        self.dmg_compatibility = compat
    }
    fn dmg_compatible(&self) -> bool {
        self.dmg_compatibility
    }
    fn access_cgb_reg(&self) -> bool {
        !self.dmg_compatible() || self.boot_rom.is_enabled()
    }
    fn access_dmg_reg(&self) -> bool {
        self.dmg_compatible() || self.boot_rom.is_enabled()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CycleStage {
    First,
    AccessOamDma,
    AccessCartridge,
    AccessTimer,
    AccessApu,
    AccessWRam,
    AccessVRam,
    AccessOam,
    AccessPpu,
    #[default]
    Last,
}

impl Context {
    fn cycle<T: IntoData>(
        &mut self,
        stage: CycleStage,
        state: CPUState,
        f: impl Fn(&mut Self) -> T,
    ) -> u8 {
        let delta_time = if self.key1.current_speed() {
            SystemTime::from_master_clocks(2)
        } else {
            SystemTime::from_master_clocks(4)
        };
        self.time += delta_time;
        let mut res = ().into_data();
        // Nothing happens while the CPU is stopped
        if matches!(state, CPUState::Stop) {
            return res;
        }
        if state.is_normal() {
            // transfer 1 or 2 bytes per cycle depending on current speed
            let hdma_transfer_bytes = if self.key1.current_speed() { 1 } else { 2 };
            // Transparently perform HDMA transfers
            while self.hdma.transfer(
                &mut self.ppu,
                &self.cartridge,
                &self.wram,
                hdma_transfer_bytes,
            ) {
                self.halt_cycle();
            }
        }
        if stage == CycleStage::First {
            res = f(self).into_data()
        }
        // if OAM DMA is active, then the PPU wont be able to read OAM
        let block_oam = if state.is_normal() {
            let active = self.oam_dma.is_active();

            if stage == CycleStage::AccessOamDma {
                res = f(self).into_data()
            } else {
                self.oam_dma
                    .cycle(&mut self.ppu, &self.cartridge, &self.wram);
            }
            active
        } else {
            false
        };
        self.ppu.set_oam_lock(block_oam);

        if stage == CycleStage::AccessCartridge {
            if matches!(
                self.oam_dma.active_memory_region(),
                Some(MemoryRegion::Cartridge)
            ) {
                log::debug!("CPU tried addressing cartridge but OAM DMA is active there");
            } else {
                res = f(self).into_data()
            }
        }
        if stage == CycleStage::AccessTimer {
            res = f(self).into_data()
        } else {
            self.cycle_timer();
        }
        {
            // APU
            if !self.is_double_speed() {
                self.apu.cycle();
            }
            if stage == CycleStage::AccessApu {
                res = f(self).into_data()
            } else {
                self.apu.cycle();
            }
        }
        if stage == CycleStage::AccessWRam {
            if matches!(
                self.oam_dma.active_memory_region(),
                Some(MemoryRegion::WRam)
            ) {
                log::debug!("CPU tried addressing WRAM but OAM DMA is active there");
            } else {
                res = f(self).into_data()
            }
        }
        {
            let ppu_mode_before = self.ppu.read_mode();
            if stage == CycleStage::AccessVRam
                || stage == CycleStage::AccessOam
                || stage == CycleStage::AccessPpu
            {
                if stage == CycleStage::AccessOam && block_oam {
                    log::debug!("CPU tried addressing OAM but OAM DMA is active");
                    self.cycle_ppu();
                } else if stage == CycleStage::AccessVRam
                    && matches!(
                        self.oam_dma.active_memory_region(),
                        Some(MemoryRegion::VRam)
                    )
                {
                    log::debug!("CPU tried addressing VRAM but OAM DMA took priority");
                    self.cycle_ppu();
                } else {
                    res = f(self).into_data()
                }
            } else {
                self.cycle_ppu();
            }
            let ppu_mode_after = self.ppu.read_mode();
            if !ppu_mode_before.is_hblank() && ppu_mode_after.is_hblank() {
                self.hdma.signal_mode_0();
            }
        }
        if stage == CycleStage::Last {
            res = f(self).into_data()
        }
        self.ppu.signal_cycle_end();
        res
    }
    fn halt_cycle(&mut self) {
        self.cycle(Default::default(), CPUState::Halt(0), |_| ());
    }
    fn generic_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(Default::default(), CPUState::Normal, f)
    }
    fn cartridge_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessCartridge, CPUState::Normal, f)
    }
    fn vram_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessVRam, CPUState::Normal, f)
    }
    fn wram_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessWRam, CPUState::Normal, f)
    }
    fn oam_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessOam, CPUState::Normal, f)
    }
    fn ppu_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessPpu, CPUState::Normal, f)
    }
    fn timer_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessTimer, CPUState::Normal, f)
    }
    fn apu_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessApu, CPUState::Normal, f)
    }
    fn oam_dma_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessOamDma, CPUState::Normal, f)
    }
}

impl CpuContext for Context {
    fn cycle_read_itrs(&mut self, addr: u16) -> (u8, InterruptFlags) {
        let data = match addr {
            0..0x100 | 0x200..0x900 if self.boot_rom.is_enabled() => {
                self.generic_cycle(|slf| slf.boot_rom.read(addr))
            }
            0x0000..0x8000 | 0xA000..0xC000 => self.cartridge_cycle(|slf| slf.cartridge.read(addr)),
            0x8000..0xA000 => self.vram_cycle(|slf| {
                let (ppu, mut ctx) = slf.ppu_ctx();
                ppu.cycle_read_vram(&mut ctx, addr)
            }),
            0xC000..0xFE00 => self.wram_cycle(|slf| slf.wram.read(addr)),
            0xFE00..0xFEA0 => self.oam_cycle(|slf| {
                let (ppu, mut ctx) = slf.ppu_ctx();
                ppu.cycle_read_oam(&mut ctx, addr)
            }),
            0xFEA0..0xFF00 => {
                log::debug!("Read into forbidden area {addr:04x}");
                self.generic_cycle(|_| ())
            }
            0xFF00 => self.generic_cycle(|slf| slf.p1.read()),
            0xFF04 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_read_div(&mut ctx)
            }),
            0xFF05 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_read_tima(&mut ctx)
            }),
            0xFF06 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_read_tma(&mut ctx)
            }),
            0xFF07 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_read_tac(&mut ctx)
            }),
            0xFF0F => self.generic_cycle(|slf| slf.interrupts),
            0xFF10 => self.apu_cycle(|slf| slf.apu.cycle_nr10_read()),
            0xFF11 => self.apu_cycle(|slf| slf.apu.cycle_nr11_read()),
            0xFF12 => self.apu_cycle(|slf| slf.apu.cycle_nr12_read()),
            // 0xFF13 => self.apu_cycle(|slf| slf.apu.cycle_nr13_read()),
            0xFF14 => self.apu_cycle(|slf| slf.apu.cycle_nr14_read()),
            0xFF16 => self.apu_cycle(|slf| slf.apu.cycle_nr21_read()),
            0xFF17 => self.apu_cycle(|slf| slf.apu.cycle_nr22_read()),
            // 0xFF18 => self.apu_cycle(|slf| slf.apu.cycle_nr23_read()),
            0xFF19 => self.apu_cycle(|slf| slf.apu.cycle_nr24_read()),
            0xFF1A => self.apu_cycle(|slf| slf.apu.cycle_nr30_read()),
            // 0xFF1B => self.apu_cycle(|slf| slf.apu.cycle_nr31_read()),
            0xFF1C => self.apu_cycle(|slf| slf.apu.cycle_nr32_read()),
            // 0xFF1D => self.apu_cycle(|slf| slf.apu.cycle_nr33_read()),
            0xFF1E => self.apu_cycle(|slf| slf.apu.cycle_nr34_read()),
            // 0xFF20 => self.apu_cycle(|slf| slf.apu.cycle_nr41_read()),
            0xFF21 => self.apu_cycle(|slf| slf.apu.cycle_nr42_read()),
            0xFF22 => self.apu_cycle(|slf| slf.apu.cycle_nr43_read()),
            0xFF23 => self.apu_cycle(|slf| slf.apu.cycle_nr44_read()),
            0xFF24 => self.apu_cycle(|slf| slf.apu.cycle_nr50_read()),
            0xFF25 => self.apu_cycle(|slf| slf.apu.cycle_nr51_read()),
            0xFF26 => self.apu_cycle(|slf| slf.apu.cycle_nr52_read()),
            0xFF30..=0xFF3F => {
                self.apu_cycle(|slf| slf.apu.cycle_pattern_ram_read((addr - 0xFF30) as u8))
            }
            0xFF40 => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.read_lcdc())),
            0xFF41 => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.read_stat())),
            0xFF42 => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.scy)),
            0xFF43 => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.scx)),
            0xFF44 => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.read_ly())),
            0xFF45 => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.lyc)),
            0xFF46 => self.oam_dma_cycle(|slf| {
                slf.oam_dma
                    .cycle_read_dma(&mut slf.ppu, &slf.cartridge, &slf.wram)
            }),
            0xFF47 if self.access_dmg_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.bgp))
            }
            0xFF48 if self.access_dmg_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.obp0))
            }
            0xFF49 if self.access_dmg_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.obp1))
            }
            0xFF4A => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.wy)),
            0xFF4B => self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.wx)),
            0xFF4D if self.access_cgb_reg() => self.generic_cycle(|slf| slf.key1.read()),
            0xFF4F if self.access_cgb_reg() => {
                self.ppu_cycle(|ppu| ppu.cycle_ppu_read(|ppu| ppu.read_vbk()))
            }
            0xFF55 if self.access_cgb_reg() => self.generic_cycle(|slf| slf.hdma.read_hdma5()),
            0xFF68 if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.bgpi))
            }
            0xFF69 if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.read_bgpd()))
            }
            0xFF6A if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.ocpi))
            }
            0xFF6B if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.read_ocpd()))
            }
            0xFF6C if self.boot_rom.is_enabled() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_read(|ppu| ppu.opri as u8 | 0xFE))
            }
            0xFF70 if self.access_cgb_reg() => self.generic_cycle(|slf| slf.wram.read_svbk()),
            0xFF76 if self.access_cgb_reg() => self.apu_cycle(|slf| slf.apu.cycle_pcm12_read()),
            0xFF77 if self.access_cgb_reg() => self.apu_cycle(|slf| slf.apu.cycle_pcm34_read()),
            0xFF80..0xFFFF => self.generic_cycle(|slf| slf.hram.read(addr)),
            0xFFFF => self.generic_cycle(|slf| slf.interrupt_enable),
            _ => {
                log::warn!("Read into unknown address {addr:04x}");
                self.generic_cycle(|_| ())
            }
        };
        (data, self.active_interrupts())
    }

    fn cycle_write_itrs(&mut self, addr: u16, data: u8) -> InterruptFlags {
        match addr {
            0x0000..0x8000 | 0xA000..0xC000 => {
                self.cartridge_cycle(|slf| slf.cartridge.write(addr, data))
            }

            0x8000..0xA000 => self.vram_cycle(|slf| {
                let (ppu, mut ctx) = slf.ppu_ctx();
                ppu.cycle_write_vram(&mut ctx, addr, data);
            }),
            0xC000..0xFE00 => self.wram_cycle(|slf| {
                slf.wram.write(addr, data);
            }),
            0xFE00..0xFEA0 => self.oam_cycle(|slf| {
                let (ppu, mut ctx) = slf.ppu_ctx();
                ppu.cycle_write_oam(&mut ctx, addr, data)
            }),
            0xFEA0..0xFF00 => {
                log::debug!("Write into forbidden area {addr:04x}");
                self.generic_cycle(|_| ())
            }
            0xFF00 => self.generic_cycle(|slf| slf.p1.write(data)),
            0xFF04 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_write_div(&mut ctx, data)
            }),
            0xFF05 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_write_tima(&mut ctx, data)
            }),
            0xFF06 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_write_tma(&mut ctx, data)
            }),
            0xFF07 => self.timer_cycle(|slf| {
                let (timer, mut ctx) = slf.timer_ctx();
                timer.cycle_write_tac(&mut ctx, data)
            }),
            0xFF0F => self.generic_cycle(|slf| slf.interrupts = (data & 0x1F).into()),
            0xFF10 => self.apu_cycle(|slf| slf.apu.cycle_nr10_write(data)),
            0xFF11 => self.apu_cycle(|slf| slf.apu.cycle_nr11_write(data)),
            0xFF12 => self.apu_cycle(|slf| slf.apu.cycle_nr12_write(data)),
            0xFF13 => self.apu_cycle(|slf| slf.apu.cycle_nr13_write(data)),
            0xFF14 => self.apu_cycle(|slf| slf.apu.cycle_nr14_write(data)),
            0xFF16 => self.apu_cycle(|slf| slf.apu.cycle_nr21_write(data)),
            0xFF17 => self.apu_cycle(|slf| slf.apu.cycle_nr22_write(data)),
            0xFF18 => self.apu_cycle(|slf| slf.apu.cycle_nr23_write(data)),
            0xFF19 => self.apu_cycle(|slf| slf.apu.cycle_nr24_write(data)),
            0xFF1A => self.apu_cycle(|slf| slf.apu.cycle_nr30_write(data)),
            0xFF1B => self.apu_cycle(|slf| slf.apu.cycle_nr31_write(data)),
            0xFF1C => self.apu_cycle(|slf| slf.apu.cycle_nr32_write(data)),
            0xFF1D => self.apu_cycle(|slf| slf.apu.cycle_nr33_write(data)),
            0xFF1E => self.apu_cycle(|slf| slf.apu.cycle_nr34_write(data)),
            0xFF20 => self.apu_cycle(|slf| slf.apu.cycle_nr41_write(data)),
            0xFF21 => self.apu_cycle(|slf| slf.apu.cycle_nr42_write(data)),
            0xFF22 => self.apu_cycle(|slf| slf.apu.cycle_nr43_write(data)),
            0xFF23 => self.apu_cycle(|slf| slf.apu.cycle_nr44_write(data)),
            0xFF24 => self.apu_cycle(|slf| slf.apu.cycle_nr50_write(data)),
            0xFF25 => self.apu_cycle(|slf| slf.apu.cycle_nr51_write(data)),
            0xFF26 => self.apu_cycle(|slf| slf.apu.cycle_nr52_write(data)),
            0xFF30..=0xFF3F => {
                self.apu_cycle(|slf| slf.apu.cycle_pattern_ram_write((addr - 0xFF30) as u8, data))
            }
            0xFF40 => self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.write_lcdc(data))),
            0xFF41 => self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.write_stat(data))),
            0xFF42 => self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.scy = data)),
            0xFF43 => self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.scx = data)),
            0xFF45 => self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.lyc = data)),
            0xFF46 => self.oam_dma_cycle(|slf| {
                slf.oam_dma
                    .cycle_write_dma(&mut slf.ppu, &slf.cartridge, &slf.wram, data)
            }),
            0xFF47 if self.access_dmg_reg() || self.boot_rom.is_enabled() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.write_bgp(data)))
            }
            0xFF48 if self.access_dmg_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.obp0 = data))
            }
            0xFF49 if self.access_dmg_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.obp1 = data))
            }
            0xFF4C if self.boot_rom.is_enabled() => {
                self.generic_cycle(|slf| slf.set_dmg_compatibility((data & 0b100) != 0))
            }
            0xFF4D if self.access_cgb_reg() => self.generic_cycle(|slf| slf.key1.write(data)),
            0xFF4F if self.access_cgb_reg() => {
                self.ppu_cycle(|ppu| ppu.cycle_ppu_write(|ppu| ppu.write_vbk(data)))
            }
            0xFF50 if self.boot_rom.is_enabled() => {
                self.boot_rom.disable();
                self.ppu.set_dmg_palette();
                ().into_data()
            }
            0xFF51 if self.access_cgb_reg() => {
                self.generic_cycle(|slf| slf.hdma.write_hdma_src_high(data))
            }
            0xFF52 if self.access_cgb_reg() => {
                self.generic_cycle(|slf| slf.hdma.write_hdma_src_low(data))
            }
            0xFF53 if self.access_cgb_reg() => {
                self.generic_cycle(|slf| slf.hdma.write_hdma_dst_high(data))
            }
            0xFF54 if self.access_cgb_reg() => {
                self.generic_cycle(|slf| slf.hdma.write_hdma_dst_low(data))
            }
            0xFF55 if self.access_cgb_reg() => {
                self.generic_cycle(|slf| slf.hdma.write_hdma5(&slf.ppu, data))
            }
            0xFF68 if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.bgpi = data))
            }
            0xFF69 if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.write_bgpd(data)))
            }
            0xFF6A if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.ocpi = data))
            }
            0xFF6B if self.access_cgb_reg() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.write_ocpd(data)))
            }
            0xFF6C if self.boot_rom.is_enabled() => {
                self.ppu_cycle(|slf| slf.cycle_ppu_write(|ppu| ppu.opri = data & 1 != 0))
            }
            0xFF70 if self.access_cgb_reg() => self.generic_cycle(|slf| slf.wram.write_svbk(data)),
            0xFF80..0xFFFF => self.generic_cycle(|slf| slf.hram.write(addr, data)),
            0xFFFF => self.generic_cycle(|slf| slf.interrupt_enable = (data & 0x1F).into()),
            _ => {
                log::warn!("Write into unknown address {addr:04x}");
                self.generic_cycle(|_| ())
            }
        };
        self.active_interrupts()
    }

    fn cycle_state_itrs(&mut self, state: CPUState) -> InterruptFlags {
        self.cycle(Default::default(), state, |_| ());
        self.active_interrupts()
    }

    fn ack_interrupt(&mut self, itr: Interrupt) {
        match itr {
            Interrupt::VBLANK => self.interrupts.set_vblank(false),
            Interrupt::LCD => self.interrupts.set_lcd(false),
            Interrupt::TIMER => self.interrupts.set_timer(false),
            Interrupt::SERIAL => self.interrupts.set_serial(false),
            Interrupt::JOYPAD => self.interrupts.set_joypad(false),
        }
    }

    fn speed_switch(&mut self) {
        self.key1.switch_speed();
    }

    fn has_pressed_input(&self) -> bool {
        self.p1.has_pressed_input()
    }
    fn has_speed_switch_armed(&self) -> bool {
        self.key1.switch_armed()
    }
    fn reset_div(&mut self) {
        self.timer.reset_div();
    }

    fn signal_event(&mut self, event: Event) {
        self.events.signal_event(event);
    }
}

pub struct TimerContextImpl<'a> {
    itrs: &'a mut InterruptFlags,
    double_speed: bool,
    apu: &'a mut Apu,
}

impl<'a> TimerContext for TimerContextImpl<'a> {
    fn signal_timer_interrupt(&mut self) {
        self.itrs.set_timer(true);
    }

    fn is_double_speed(&self) -> bool {
        self.double_speed
    }

    fn signal_div_apu_event(&mut self) {
        self.apu.div_apu_tick();
    }
}

impl Context {
    fn timer_ctx(&'_ mut self) -> (&'_ mut Timer, TimerContextImpl<'_>) {
        let ctx = TimerContextImpl {
            itrs: &mut self.interrupts,
            double_speed: self.key1.current_speed(),
            apu: &mut self.apu,
        };
        (&mut self.timer, ctx)
    }
    fn cycle_timer(&mut self) {
        let (timer, mut ctx) = self.timer_ctx();
        timer.cycle(&mut ctx);
    }
}

pub struct PpuContextImpl<'a> {
    events: &'a mut Events,
    itrs: &'a mut InterruptFlags,
    double_speed: bool,
    dmg_compatible: bool,
}

impl<'a> PpuContext for PpuContextImpl<'a> {
    fn signal_vblank_interrupt(&mut self) {
        self.events.signal_vblank();
        self.itrs.set_vblank(true);
    }

    fn signal_lcd_interrupt(&mut self) {
        self.itrs.set_lcd(true);
    }

    fn is_double_speed(&self) -> bool {
        self.double_speed
    }

    fn is_dmg_compatible(&self) -> bool {
        self.dmg_compatible
    }
}

impl Context {
    fn ppu_ctx(&'_ mut self) -> (&'_ mut Ppu, PpuContextImpl<'_>) {
        let dmg_compatible = self.dmg_compatible();
        let ctx = PpuContextImpl {
            events: &mut self.events,
            itrs: &mut self.interrupts,
            double_speed: self.key1.current_speed(),
            dmg_compatible,
        };
        (&mut self.ppu, ctx)
    }
    fn cycle_ppu(&mut self) {
        let (ppu, mut ctx) = self.ppu_ctx();
        ppu.cycle(&mut ctx);
    }
    fn cycle_ppu_read(&mut self, f: impl FnOnce(&Ppu) -> u8) -> u8 {
        let (ppu, mut ctx) = self.ppu_ctx();
        ppu.cycle_read(&mut ctx, f)
    }
    fn cycle_ppu_write(&mut self, f: impl FnOnce(&mut Ppu)) {
        let (ppu, mut ctx) = self.ppu_ctx();
        ppu.cycle_write(&mut ctx, f)
    }
}

impl Context {
    pub fn debug_read_memory(&self, addr: u16) -> Option<u8> {
        let data = match addr {
            0..0x100 | 0x200..0x900 if self.boot_rom.is_enabled() => self.boot_rom.read(addr),
            0x0000..0x8000 | 0xA000..0xC000 => self.cartridge.read(addr),
            0x8000..0xA000 => self.ppu.read_vram(addr),
            0xC000..0xFE00 => self.wram.read(addr),
            0xFE00..0xFEA0 => self.ppu.read_oam(addr),
            0xFF80..=0xFFFE => self.hram.read(addr),
            _ => return None,
        };
        Some(data)
    }
    pub fn get_ppu(&self) -> &Ppu {
        &self.ppu
    }
    pub fn get_apu(&self) -> &Apu {
        &self.apu
    }
}

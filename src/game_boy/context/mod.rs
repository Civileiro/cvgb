use hdma::Hdma;
use hram::HighRam;
use interrupts::{Interrupt, InterruptFlags};
use key1::Key1;
use oam_dma::OamDma;
use p1::P1;
use ppu::{Ppu, PpuContext};
use timer::Timer;
use wram::WorkRam;

use super::{
    BootRom,
    cartridge::Cartridge,
    cpu::{CPUState, CpuContext},
    events::Events,
    input::Input,
    time::SystemTime,
};

mod hdma;
mod hram;
pub mod interrupts;
mod key1;
mod oam_dma;
mod p1;
mod ppu;
mod timer;
mod wram;
pub use ppu::VideoBuffer;

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
            key1: Default::default(),
            hram: Default::default(),
            interrupts: Default::default(),
            interrupt_enable: Default::default(),
        }
    }
    pub fn system_time(&self) -> SystemTime {
        self.time
    }
    pub fn set_input(&mut self, input: Input) {
        if self.p1.set_input(input) {
            self.interrupts.set_joypad(true);
        }
    }
    pub fn fetch_clear_events(&mut self) -> Events {
        let res = self.events;
        self.events = Events::new();
        res
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
    fn active_interrupts(&self) -> InterruptFlags {
        (Into::<u8>::into(self.interrupts) & Into::<u8>::into(self.interrupt_enable)).into()
    }
    fn is_double_speed_cycle(&self) -> bool {
        self.key1.current_speed() && (self.time.cycles() % 4 != 0)
    }
    fn set_dmg_compatibility(&mut self, compat: bool) {
        self.dmg_compatibility = compat
    }
    fn dmg_compatible(&self) -> bool {
        self.dmg_compatibility
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CycleStage {
    Before,
    AccessOamDma,
    AccessCartridge,
    AccessTimer,
    AccessWRam,
    AccessVRam,
    AccessOam,
    #[default]
    BeforeHdma,
    AfterHdma,
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
            self.timer.cycle(&mut self.interrupts);
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
            if stage == CycleStage::AccessVRam || stage == CycleStage::AccessOam {
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
        if stage == CycleStage::BeforeHdma {
            res = f(self).into_data()
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
                self.cycle(Default::default(), CPUState::Halt(0), |_| ());
            }
        }
        if stage == CycleStage::AfterHdma {
            res = f(self).into_data()
        }
        res
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
    fn timer_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessTimer, CPUState::Normal, f)
    }
    fn oam_dma_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AccessOamDma, CPUState::Normal, f)
    }
    fn after_hdma_cycle<T: IntoData>(&mut self, f: impl Fn(&mut Self) -> T) -> u8 {
        self.cycle(CycleStage::AfterHdma, CPUState::Normal, f)
    }
}

impl CpuContext for Context {
    fn cycle_read_itrs(&mut self, addr: u16) -> (u8, InterruptFlags) {
        let data = match addr {
            0..0x100 | 0x200.. if self.boot_rom.is_enabled() => {
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
            0xFF04 => self.timer_cycle(|slf| slf.timer.cycle_read_div(&mut slf.interrupts)),
            0xFF05 => self.timer_cycle(|slf| slf.timer.cycle_read_tima(&mut slf.interrupts)),
            0xFF06 => self.timer_cycle(|slf| slf.timer.cycle_read_tma(&mut slf.interrupts)),
            0xFF07 => self.timer_cycle(|slf| slf.timer.cycle_read_tac(&mut slf.interrupts)),
            0xFF0F => self.generic_cycle(|slf| slf.interrupts),
            0xFF40 => self.generic_cycle(|slf| slf.ppu.read_lcdc()),
            0xFF41 => self.generic_cycle(|slf| slf.ppu.read_stat()),
            0xFF46 => self.oam_dma_cycle(|slf| {
                slf.oam_dma
                    .cycle_read_dma(&mut slf.ppu, &slf.cartridge, &slf.wram)
            }),
            0xFF4D if !self.dmg_compatible() => self.generic_cycle(|slf| slf.key1.read()),
            0xFF55 if !self.dmg_compatible() => self.after_hdma_cycle(|slf| slf.hdma.read_hdma5()),
            0xFF70 if !self.dmg_compatible() => self.generic_cycle(|slf| slf.wram.read_svbk()),
            0xFF80..0xFFFF => self.generic_cycle(|slf| slf.hram.read(addr)),
            0xFFFF => self.generic_cycle(|slf| slf.interrupt_enable),
            _ => self.generic_cycle(|_| ()),
        };
        (data, self.active_interrupts())
    }

    fn cycle_write_itrs(&mut self, addr: u16, data: u8) -> InterruptFlags {
        todo!()
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

    fn has_interrupt(&mut self) -> bool {
        self.interrupts.has_interrupt()
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
}

pub struct PpuContextImpl<'a> {
    events: &'a mut Events,
    itrs: &'a mut InterruptFlags,
    double_speed: bool,
}

impl<'a> PpuContext for PpuContextImpl<'a> {
    fn signal_lcd_interrupt(&mut self) {
        self.itrs.set_lcd(true);
    }

    fn signal_frame_ready(&mut self) {
        todo!()
    }

    fn is_double_speed(&self) -> bool {
        self.double_speed
    }
}

impl Context {
    fn ppu_ctx(&mut self) -> (&mut Ppu, PpuContextImpl) {
        let ctx = PpuContextImpl {
            events: &mut self.events,
            itrs: &mut self.interrupts,
            double_speed: self.key1.current_speed(),
        };
        (&mut self.ppu, ctx)
    }
    fn cycle_ppu(&mut self) {
        let (ppu, mut ctx) = self.ppu_ctx();
        ppu.cycle(&mut ctx);
    }
}

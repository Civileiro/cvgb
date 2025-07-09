mod boot_rom;
mod bus;
mod cartridge;
mod constants;
mod core;
mod dma;
mod input;
mod instruction;
mod memory;
mod ppu;
mod render;
mod timer;
mod variant;
mod wram;

use boot_rom::BootRom;
pub use bus::{Addressable, Bus};
pub use cartridge::Cartridge;
use core::{CPU, CPUIORegisters};
use dma::Dma;
use input::JoypadInput;
pub use memory::Memory;
use ppu::{Oam, Ppu, VRam, VRamBank};
use std::{
    ops::Deref,
    rc::{Rc, Weak},
};
use timer::Timer;
use variant::GameBoyVariant;
use wram::{WRam, WRamBank};

pub use constants::{OUTPUT_HEIGHT, OUTPUT_WIDTH};

pub struct GameBoy {
    variant: Rc<GameBoyVariant>,
    cpu: Rc<CPU>,
    cpu_io: Rc<CPUIORegisters>,
    boot_rom: Rc<BootRom>,
    cartridge: Rc<Cartridge>,
    bus: Rc<Bus>,
    wram: Rc<WRam>,
    wram_bank: Rc<WRamBank>,
    ppu: Rc<Ppu>,
    vram: Rc<VRam>,
    vram_bank: Rc<VRamBank>,
    hram: Rc<Memory<Box<[u8]>>>,
    timer: Rc<Timer>,
    pub input: Rc<JoypadInput>,
    dma: Rc<Dma>,
}

impl GameBoy {
    pub fn new(rom: Box<[u8]>) -> Self {
        let bus = Rc::new(Bus::new());
        let variant = Rc::new(GameBoyVariant::DMG);

        let cartridge = Rc::new(Cartridge::from_rom(rom, Rc::clone(&bus)).unwrap());
        let boot_rom = Rc::new(BootRom::new(variant.deref().clone()));
        let cpu = Rc::new(CPU::new(Rc::clone(&bus)));
        let cpu_io = Rc::new(CPUIORegisters::new(
            Rc::clone(&variant),
            Rc::clone(&boot_rom),
            Rc::clone(&cpu),
        ));
        let ppu = Rc::new(Ppu::new(Rc::clone(&variant), Rc::clone(&bus)));
        let timer = Rc::new(Timer::new(variant.deref().clone(), Rc::clone(&cpu_io)));
        let input = Rc::new(JoypadInput::new());
        let dma = Rc::new(Dma::new(variant.deref().clone(), Rc::clone(&bus)));

        let vram = Rc::new(VRam::new(Rc::clone(&ppu)));
        let vram_bank = Rc::new(VRamBank::new(Rc::clone(&vram)));
        let oam = Rc::new(Oam::new(Rc::clone(&ppu)));
        let wram = Rc::new(WRam::new(variant.deref().clone()));
        let wram_bank = Rc::new(WRamBank::new(Rc::clone(&variant), Rc::clone(&wram)));
        let hram = Rc::new(Memory::new(vec![0u8; 0x7F].into_boxed_slice(), 0x7E));

        bus.plug_memory_with_priority(
            Rc::downgrade(&boot_rom) as Weak<dyn Addressable>,
            0x0000..=0x08FF,
            10,
        );
        bus.plug_memory(
            Rc::downgrade(&cartridge) as Weak<dyn Addressable>,
            0x0000..=0xBFFF,
        );
        bus.plug_memory(
            Rc::downgrade(&vram) as Weak<dyn Addressable>,
            0x8000..=0x9FFF,
        );
        bus.plug_memory(
            Rc::downgrade(&wram) as Weak<dyn Addressable>,
            0xC000..=0xDFFF,
        );
        bus.plug_memory(
            Rc::downgrade(&wram) as Weak<dyn Addressable>,
            0xE000..=0xFDFF,
        );
        bus.plug_memory(
            Rc::downgrade(&oam) as Weak<dyn Addressable>,
            0xFE00..=0xFE9F,
        );
        bus.plug_memory(
            Rc::downgrade(&input) as Weak<dyn Addressable>,
            0xFF00..=0xFF00,
        );
        bus.plug_memory(
            Rc::downgrade(&timer) as Weak<dyn Addressable>,
            0xFF04..=0xFF07,
        );
        bus.plug_memory(
            Rc::downgrade(&dma) as Weak<dyn Addressable>,
            0xFF46..=0xFF46,
        );
        bus.plug_memory(
            Rc::downgrade(&vram_bank) as Weak<dyn Addressable>,
            0xFF4F..=0xFF4F,
        );
        bus.plug_memory(
            Rc::downgrade(&wram_bank) as Weak<dyn Addressable>,
            0xFF70..=0xFF70,
        );
        bus.plug_memory(
            Rc::downgrade(&ppu) as Weak<dyn Addressable>,
            0xFF00..=0xFFFF,
        );
        bus.plug_memory(
            Rc::downgrade(&cpu_io) as Weak<dyn Addressable>,
            0xFF00..=0xFFFF,
        );
        bus.plug_memory(
            Rc::downgrade(&hram) as Weak<dyn Addressable>,
            0xFF80..=0xFFFE,
        );

        Self {
            variant,
            cpu,
            cpu_io,
            boot_rom,
            cartridge,
            bus,
            wram,
            wram_bank,
            ppu,
            vram,
            vram_bank,
            hram,
            timer,
            input,
            dma,
        }
    }
    pub fn step(&self) {
        // Frequency = cpu_speed * 4194304 Hz
        let cpu_speed = if self.cpu.is_double_speed() && !self.cpu.is_stopped() {
            2
        } else {
            1
        };
        for _ in 0..cpu_speed {
            self.cpu.clock();
            if !self.cpu.is_stopped() {
                self.timer.clock();
                self.dma.clock();
            }
        }
        self.ppu.clock();
    }
    pub fn step_instruction(&self) {
        if !self.cpu.is_executing_instruction() {
            self.step();
        }
        while self.cpu.is_executing_instruction() {
            self.step();
        }
    }
    pub fn frame_is_ready(&self) -> bool {
        self.ppu.frame_is_ready()
    }
    pub fn step_frame(&self) {
        while !self.ppu.frame_is_ready() {
            self.step();
        }
    }
}

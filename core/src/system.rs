use crate::cpu::registers::{Flags, Registers};

use super::{
    Cartridge, Input, Rom,
    cartridge::CartridgeParseError,
    context::{AudioOutput, Context, VideoBuffer},
    cpu::Cpu,
    events::Events,
    time::SystemTime,
};

#[derive(Debug)]
pub struct System {
    cpu: Cpu,
    context: Context,
}

impl System {
    pub fn new(cartridge: Cartridge, bootrom_enabled: bool) -> Self {
        Self {
            cpu: if bootrom_enabled {
                Default::default()
            } else {
                let regs = Registers {
                    a: 0x11,
                    f: Flags::default().with_z(true),
                    b: 0x00,
                    c: 0x00,
                    d: 0xFF,
                    e: 0x56,
                    h: 0x00,
                    l: 0x0D,
                    pc: 0x0100,
                    sp: 0xFFFE,
                };
                Cpu::with_registers(regs)
            },
            context: Context::new(cartridge, bootrom_enabled),
        }
    }
    pub fn get_cpu(&self) -> &Cpu {
        &self.cpu
    }
    pub fn set_breakpoint_addr(&mut self, addr: Option<u16>) {
        self.cpu.set_breakpoint_addr(addr);
    }
    pub fn get_context(&self) -> &Context {
        &self.context
    }
    pub fn get_cpu_context(&self) -> (&Cpu, &Context) {
        (&self.cpu, &self.context)
    }
    pub fn step(&mut self) -> Events {
        self.cpu.step(&mut self.context);
        self.context.fetch_clear_events()
    }
    pub fn time(&self) -> SystemTime {
        self.context.system_time()
    }
    pub fn advance(&mut self, delta: SystemTime) -> (Events, SystemTime) {
        puffin::profile_function!();
        let target_time = self.time() + delta;
        let start_time = self.time();
        let mut events: Events = Default::default();
        log::debug!("Trying to advance system by {delta}");
        while self.time() < target_time && events.is_empty() {
            events = self.step();
        }
        let elapsed_time = self.time() - start_time;
        log::debug!("Advanced system by {elapsed_time}");
        (events, elapsed_time)
    }
    pub fn press_key(&mut self, input: Input) {
        self.context.press_key(input);
    }
    pub fn unpress_key(&mut self, input: Input) {
        self.context.unpress_key(input);
    }
    pub fn get_video_buffer(&self) -> &VideoBuffer {
        self.context.get_video_buffer()
    }
    pub fn get_audio_output(&mut self) -> AudioOutput {
        self.context.get_audio_output()
    }
    pub fn get_serial_port_output(&mut self) -> Box<[u8]> {
        self.context.get_serial_port_output()
    }
}

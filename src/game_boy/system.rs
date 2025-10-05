use std::{cell::RefCell, rc::Rc};

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
    pub fn new(rom: Rom) -> Result<Self, CartridgeParseError> {
        let cartridge = Cartridge::from_rom(rom)?;
        Ok(Self {
            cpu: Default::default(),
            context: Context::new(cartridge),
        })
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
    pub fn set_input(&mut self, input: Input) {
        self.context.set_input(input);
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
}

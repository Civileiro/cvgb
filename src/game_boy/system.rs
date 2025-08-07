use super::{
    Cartridge, Input, Rom,
    cartridge::{self, CartridgeParseError},
    context::Context,
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
    pub fn now(rom: Rom) -> Result<Self, CartridgeParseError> {
        let cartridge = Cartridge::from_rom(rom)?;
        Ok(Self {
            cpu: Default::default(),
            context: Context::new(cartridge),
        })
    }

    pub fn step(&mut self) -> Events {
        self.cpu.step(&mut self.context);
        self.context.fetch_clear_events()
    }
    pub fn time(&self) -> SystemTime {
        self.context.system_time()
    }
    pub fn advance(&mut self, delta: SystemTime) -> (Events, SystemTime) {
        let target_time = self.time() + delta;
        let start_time = self.time();
        let mut events = Events::new();
        while self.time() < target_time && !events.is_empty() {
            events = self.step();
        }
        let elapsed_time = self.time() - start_time;
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
}

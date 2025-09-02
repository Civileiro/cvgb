#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Events {
    vblank: bool,
    breakpoint: bool,
}

pub enum Event {
    VBlank,
    Breakpoint,
}

impl Events {
    pub fn empty(&self) -> bool {
        *self == Self::default()
    }
    pub fn signal_event(&mut self, event: Event) {
        match event {
            Event::VBlank => self.signal_vblank(),
            Event::Breakpoint => self.signal_breakpoint(),
        }
    }
    pub fn signal_vblank(&mut self) {
        self.vblank = true;
    }
    pub fn has_vblank(&self) -> bool {
        self.vblank
    }
    pub fn signal_breakpoint(&mut self) {
        self.breakpoint = true;
    }
    pub fn has_breakpoint(&self) -> bool {
        self.breakpoint
    }
}

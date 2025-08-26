#[derive(Debug, Clone, Default)]
pub struct Events {
    vblank: bool,
    // breakpoint: bool,
}

impl Events {
    pub fn empty(&self) -> bool {
        !self.has_vblank()
    }
    pub fn signal_vblank(&mut self) {
        self.vblank = true;
    }
    pub fn has_vblank(&self) -> bool {
        self.vblank
    }
}

const DELAY_SIZE: usize = 2;

#[derive(Debug, Default)]
pub struct SampleDelay {
    read_queue: [bool; DELAY_SIZE],
}

impl SampleDelay {
    pub fn read_now(&mut self) -> bool {
        self.read_queue.rotate_left(1);
        core::mem::replace(&mut self.read_queue[DELAY_SIZE - 1], false)
    }
    pub fn add_read(&mut self) {
        self.read_queue[DELAY_SIZE - 1] = true;
    }
    pub fn flush(&mut self) {
        self.read_queue = Default::default()
    }
}

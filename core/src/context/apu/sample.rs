const READ_QUEUE_SIZE: usize = 2;

#[derive(Debug, Default)]
pub struct SampleQueue<T: Copy> {
    read_queue: [Option<T>; READ_QUEUE_SIZE],
    read_buffer: T,
}

impl<T: Copy> SampleQueue<T> {
    pub fn tick(&mut self) {
        self.read_queue.rotate_left(1);
        if let Some(sample) = self.read_queue[READ_QUEUE_SIZE - 1].take() {
            self.read_buffer = sample;
        }
    }
    pub fn update_sample(&mut self, sample: T) {
        self.read_queue[READ_QUEUE_SIZE - 1] = Some(sample);
    }
    pub fn get_sample(&self) -> T {
        self.read_buffer
    }
}

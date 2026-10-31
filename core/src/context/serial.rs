#[derive(Debug, Default)]
pub struct SerialPort {
    serial_buffer: u8,
    counter: u16,
    high_speed: bool,
    progress: u8,
    transfering: bool,

    output_buffer: Vec<u8>,
    clock_select: bool,
}

impl SerialPort {
    fn tick(&mut self) -> bool {
        self.counter += 1;
        let overflow = self.counter == if self.high_speed { 16 } else { 512 };
        self.counter = 0;
        overflow
    }
    pub fn cycle(&mut self, ctx: &mut impl SerialContext) {
        if !self.transfering || !self.tick() {
            return;
        }
        self.serial_buffer <<= 1; // TODO: get input data
        self.progress += 1;
        if self.progress != 8 {
            return;
        }
        self.transfering = false;
        ctx.signal_serial_interrupt();
    }
    pub fn cycle_read_serial_buffer(&mut self, ctx: &mut impl SerialContext) -> u8 {
        self.cycle(ctx);
        self.serial_buffer
    }
    pub fn cycle_write_serial_buffer(&mut self, ctx: &mut impl SerialContext, data: u8) {
        self.cycle(ctx);
        self.serial_buffer = data
    }
    pub fn cycle_read_serial_control(&mut self, ctx: &mut impl SerialContext) -> u8 {
        self.cycle(ctx);
        (self.transfering as u8) << 7 | 0b0111_1110 | self.clock_select as u8
    }
    pub fn cycle_write_serial_control(&mut self, ctx: &mut impl SerialContext, data: u8) {
        self.cycle(ctx);
        let enable = ((data >> 7) & 1) != 0;
        let clock_speed = ((data >> 1) & 1) != 0;
        self.clock_select = (data & 1) != 0;
        if enable {
            self.transfering = true;
            self.counter = 0;
            self.high_speed = clock_speed && !ctx.is_dmg_compatible();
            self.progress = 0;

            self.output_buffer.push(self.serial_buffer);
        }
    }
    pub fn take_serial_output(&mut self) -> Box<[u8]> {
        std::mem::take(&mut self.output_buffer).into_boxed_slice()
    }
}

pub trait SerialContext {
    fn signal_serial_interrupt(&mut self);
    fn is_dmg_compatible(&self) -> bool;
}

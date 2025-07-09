use std::{
    cell::{Cell, Ref, RefCell},
    rc::{Rc, Weak},
};

use modular_bitfield::prelude::*;

use crate::gb::constants::{DIV_ADDR, INTERRUPT_ENABLE_ADDR, INTERRUPTS_ADDR};

use super::{
    Addressable,
    boot_rom::BootRom,
    bus::{AddressingError, Bus},
    constants::P1_ADDR,
    instruction::Instruction,
    variant::GameBoyVariant,
};

/// SM83 Core
/// Holds teh entire CPU state
pub struct Core {
    /// Current cycle count
    cycles: u64,
    /// Instruction cycle
    instruction_cycle: u8,
    /// Registers
    r: [u16; 6],
    /// Address Bus
    bus: Rc<Bus>,
    /// Interrupt master enable flag
    ime: bool,
    /// EI delay
    ei_delay: bool,
    /// CPU halt mode
    halt: bool,
    /// HALT bug emulation flag
    halt_bug: bool,
    /// CPU stop mode
    stop: bool,
    /// Speed change period
    speed_change_timer: u32,
    auto_leave_halt_timer: u32,
    //// CPU IO registers
    interrupts: u8,
    current_speed: bool,
    switch_armed: bool,
    interrupt_enable: u8,
}

impl Core {
    pub fn new(bus: Rc<Bus>) -> Self {
        Self {
            cycles: 0,
            instruction_cycle: 0,
            r: [0; 6],
            bus,
            ime: false,
            ei_delay: false,
            halt: false,
            halt_bug: false,
            stop: false,
            speed_change_timer: 0,
            auto_leave_halt_timer: 0,
            interrupts: 0,
            current_speed: false,
            switch_armed: false,
            interrupt_enable: 0,
        }
    }
    pub fn get_af(&self) -> u16 {
        self.r[0]
    }
    pub fn set_af(&mut self, data: u16) {
        self.r[0] = data
    }
    pub fn get_bc(&self) -> u16 {
        self.r[1]
    }
    pub fn set_bc(&mut self, data: u16) {
        self.r[1] = data
    }
    pub fn get_de(&self) -> u16 {
        self.r[2]
    }
    pub fn set_de(&mut self, data: u16) {
        self.r[2] = data
    }
    pub fn get_hl(&self) -> u16 {
        self.r[3]
    }
    pub fn set_hl(&mut self, data: u16) {
        self.r[3] = data
    }
    pub fn get_a(&self) -> u8 {
        (self.get_af() >> 8) as u8
    }
    pub fn set_a(&mut self, data: u8) {
        self.set_af(((data as u16) << 8) | self.get_f() as u16)
    }
    pub fn get_f(&self) -> u8 {
        self.get_af() as u8
    }
    pub fn set_f(&mut self, data: u8) {
        self.set_af(((self.get_a() as u16) << 8) | (data as u16))
    }
    pub fn get_flags(&self) -> Flags {
        Flags::from_bytes([self.get_af() as u8])
    }
    pub fn set_flags(&mut self, data: Flags) {
        self.set_f(u8::from_bytes(data.into_bytes()[0]).unwrap())
    }
    pub fn get_b(&self) -> u8 {
        (self.get_bc() >> 8) as u8
    }
    pub fn set_b(&mut self, data: u8) {
        self.set_bc(((data as u16) << 8) | self.get_c() as u16);
    }
    pub fn get_c(&self) -> u8 {
        self.get_bc() as u8
    }
    pub fn set_c(&mut self, data: u8) {
        self.set_bc(((self.get_b() as u16) << 8) | (data as u16))
    }
    pub fn get_d(&self) -> u8 {
        (self.get_de() >> 8) as u8
    }
    pub fn set_d(&mut self, data: u8) {
        self.set_de(((data as u16) << 8) | self.get_e() as u16)
    }
    pub fn get_e(&self) -> u8 {
        self.get_de() as u8
    }
    pub fn set_e(&mut self, data: u8) {
        self.set_de(((self.get_d() as u16) << 8) | (data as u16))
    }
    pub fn get_h(&self) -> u8 {
        (self.get_hl() >> 8) as u8
    }
    pub fn set_h(&mut self, data: u8) {
        self.set_hl(((data as u16) << 8) | self.get_l() as u16)
    }
    pub fn get_l(&self) -> u8 {
        self.get_hl() as u8
    }
    pub fn set_l(&mut self, data: u8) {
        self.set_hl(((self.get_h() as u16) << 8) | (data as u16))
    }
    pub fn get_z_flag(&self) -> bool {
        self.get_flags().z()
    }
    pub fn set_z_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_z(data))
    }
    pub fn get_n_flag(&self) -> bool {
        self.get_flags().n()
    }
    pub fn set_n_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_n(data))
    }
    pub fn get_h_flag(&self) -> bool {
        self.get_flags().h()
    }
    pub fn set_h_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_h(data))
    }
    pub fn get_c_flag(&self) -> bool {
        self.get_flags().c()
    }
    pub fn set_c_flag(&mut self, data: bool) {
        self.set_flags(self.get_flags().with_c(data))
    }
    /// Read stack pointer (register 4)
    pub fn get_sp(&self) -> u16 {
        self.r[4]
    }
    pub fn set_sp(&mut self, data: u16) {
        self.r[4] = data
    }
    /// Read program counter (register 5)
    pub fn get_pc(&self) -> u16 {
        self.r[5]
    }
    pub fn set_pc(&mut self, data: u16) {
        self.r[5] = data
    }
    pub fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }
    pub fn write(&self, addr: u16, data: u8) {
        self.bus.write(addr, data)
    }
    pub fn ei(&mut self) {
        self.ei_delay = true;
    }
    pub fn ei_instantly(&mut self) {
        self.ime = true;
    }
    pub fn di(&mut self) {
        self.ime = false;
    }
    pub fn get_ime(&self) -> bool {
        self.ime
    }
    pub fn halt_bug(&self) -> bool {
        self.halt_bug
    }
    pub fn set_halt(&mut self) {
        self.halt = true
    }
    pub fn reset_halt(&mut self) {
        self.halt = false;
        self.auto_leave_halt_timer = 0;
    }
    pub fn is_stopped(&self) -> bool {
        self.stop || self.speed_change_timer > 0
    }
    pub fn switch_speed(&mut self) {
        self.current_speed = !self.current_speed;
    }
    pub fn start_speed_switch(&mut self) {
        self.speed_change_timer = 2050;
        self.switch_armed = false;
        if self.halt {
            self.auto_leave_halt_timer = 0x20000;
        }
    }
    pub fn is_double_speed(&self) -> bool {
        self.current_speed && self.speed_change_timer == 0
    }

    pub fn clock(&mut self) {
        if self.speed_change_timer > 0 {
            self.speed_change_timer -= 1;
            if self.speed_change_timer == 0 {
                self.switch_speed();
                self.stop = false
            } else {
                return;
            }
        }
        let p1 = self.bus.read(P1_ADDR);
        let input_is_held = (p1 & 0x0F) != 0x0F;
        if input_is_held {
            self.stop = false
        }
        if self.stop {
            return;
        }
        if self.auto_leave_halt_timer > 0 {
            self.auto_leave_halt_timer -= 1;
            if self.auto_leave_halt_timer == 0 {
                self.reset_halt();
            }
        }

        if self.instruction_cycle == 0 {
            let interrupt_enable = InterruptFlags::from_bytes([self.interrupt_enable]);
            let mut interrupts_flag = InterruptFlags::from_bytes([self.interrupts]);
            let interrupt_requests =
                InterruptFlags::from_bytes([interrupt_enable.bytes[0] & interrupts_flag.bytes[0]]);
            let interrupt_is_pending = interrupt_requests.bytes[0] != 0;
            // Pending interrupts unpause the cpu
            if interrupt_is_pending {
                self.reset_halt();
            }
            if self.ime && interrupt_is_pending {
                if self.halt_bug {
                    // last instructions were EI followed by HALT, which causes
                    // the handler to return to the HALT instruction instead
                    // of the next one
                    self.set_pc(self.get_pc().wrapping_sub(1));
                    self.halt_bug = false
                }
                // IME is disabled until the handler usually calls RETI
                self.ime = false;
                // get handler address in order of priority
                let handler_addr = if interrupt_requests.vblank() {
                    interrupts_flag.set_vblank(false);
                    0x40
                } else if interrupt_requests.lcd() {
                    interrupts_flag.set_lcd(false);
                    0x48
                } else if interrupt_requests.timer() {
                    interrupts_flag.set_timer(false);
                    0x50
                } else if interrupt_requests.serial() {
                    interrupts_flag.set_serial(false);
                    0x58
                } else if interrupt_requests.joypad() {
                    interrupts_flag.set_joypad(false);
                    0x60
                } else {
                    unreachable!()
                };
                self.interrupts = interrupts_flag.bytes[0];
                // calling the handler is identical to a CALL instruction
                Instruction::CALLimm {
                    imm16: handler_addr,
                }
                .execute(self);
                // takes 5 cycles to call the interrupt handler
                self.instruction_cycle = 5;
            } else if !self.halt {
                if self.ei_delay {
                    self.ime = true;
                    self.ei_delay = false;
                }
                let Ok(instruction) = Instruction::decode(self) else {
                    unimplemented!()
                };
                self.set_pc(self.get_pc().wrapping_add(instruction.size() as u16));
                if self.halt_bug {
                    // If there's a halt bug happening, the instruction was decoded by
                    // repeating the first byte because the pc failed to increment once
                    // this covers the case of repeated byte reads and self-returning jumps
                    self.set_pc(self.get_pc().wrapping_sub(1));
                    self.halt_bug = false
                }

                // In the current branch IME was 0, and if there are interrupts
                // pending after executing HALT, the halt bug happens
                if interrupt_is_pending && matches!(instruction, Instruction::HALT) {
                    self.halt_bug = true;
                }
                if !matches!(instruction, Instruction::STOP) {
                    self.instruction_cycle = instruction.execute(self);
                } else {
                    // STOP clowfiesta
                    self.instruction_cycle = 1;
                    // so far the STOP incremented the pc by 2, decrement it if necessary
                    if input_is_held {
                        if interrupt_is_pending {
                            self.set_pc(self.get_pc().wrapping_sub(1));
                        } else {
                            self.set_halt();
                        }
                    } else if !self.switch_armed {
                        if interrupt_is_pending {
                            self.set_pc(self.get_pc().wrapping_sub(1));
                            self.stop = true;
                            self.bus.write(DIV_ADDR, 0);
                        } else {
                            self.stop = true;
                            self.bus.write(DIV_ADDR, 0);
                        }
                    } else if interrupt_is_pending {
                        if self.ime {
                            self.set_pc(self.get_pc().wrapping_sub(1));
                            self.bus.write(DIV_ADDR, 0);
                            self.start_speed_switch();
                        } else {
                            panic!("STOP instruction CPU glitch reached");
                        }
                    } else {
                        self.set_halt();
                        self.bus.write(DIV_ADDR, 0);
                        self.start_speed_switch();
                    }
                }
            }
            assert!(self.halt || self.instruction_cycle != 0);
        }
        self.cycles += 1;
        self.instruction_cycle -= 1;
    }

    pub fn is_executing_instruction(&self) -> bool {
        self.instruction_cycle != 0
    }
}

#[bitfield(bits = 8)]
pub struct Flags {
    #[skip]
    __: B4,
    pub c: bool,
    pub h: bool,
    pub n: bool,
    pub z: bool,
}

#[bitfield(bits = 8)]
pub struct InterruptFlags {
    pub vblank: bool,
    pub lcd: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
    #[skip]
    __: B3,
}

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    core: RefCell<Core>,
}

impl CPU {
    pub fn new(bus: Rc<Bus>) -> Self {
        Self {
            core: RefCell::new(Core::new(bus)),
        }
    }
    pub fn clock(&self) {
        self.core.borrow_mut().clock();
    }
    pub fn is_stopped(&self) -> bool {
        self.core.borrow().is_stopped()
    }
    pub fn is_executing_instruction(&self) -> bool {
        self.core.borrow().is_executing_instruction()
    }
    pub fn is_double_speed(&self) -> bool {
        self.core.borrow().is_double_speed()
    }
    pub fn core(&self) -> Ref<Core> {
        self.core.borrow()
    }
}

pub struct CPUIORegisters {
    variant: Rc<GameBoyVariant>,
    boot_rom: Rc<BootRom>,
    cpu: Rc<CPU>,
}

impl CPUIORegisters {
    pub fn new(variant: Rc<GameBoyVariant>, boot_rom: Rc<BootRom>, cpu: Rc<CPU>) -> Self {
        Self {
            variant,
            boot_rom,
            cpu,
        }
    }
    pub fn set_boot_rom_lock(&self) {
        self.boot_rom.disable();
    }
    pub fn get_interrupts(&self) -> u8 {
        self.cpu.core.borrow().interrupts | 0b11100000
    }
    pub fn set_interrupts(&self, data: u8) {
        self.cpu.core.borrow_mut().interrupts = data
    }
    pub fn get_key0(&self) -> u8 {
        if !self.variant.is_color_variant() || !self.boot_rom.is_enabled() {
            0xFF
        } else {
            ((self.variant.is_dmg_compatible() as u8) << 2) | 0b11111011
        }
    }
    pub fn set_key0(&self, data: u8) {
        if self.variant.is_color_variant() && self.boot_rom.is_enabled() {
            let dmg_compatibility = data & 0b100 != 0;
            if dmg_compatibility {
                self.variant.set_dmg_compatibility();
            }
        }
    }
    pub fn get_key1(&self) -> u8 {
        if !self.variant.is_color_variant() {
            0xFF
        } else {
            let core = self.cpu.core.borrow();
            ((core.current_speed as u8) << 7) | 0b01111110 | core.switch_armed as u8
        }
    }
    pub fn set_key1(&self, data: u8) {
        if self.variant.is_color_variant() {
            self.cpu.core.borrow_mut().switch_armed = data & 1 != 0
        }
    }
    pub fn get_interrupt_enable(&self) -> u8 {
        self.cpu.core.borrow().interrupt_enable
    }
    pub fn set_interrupt_enable(&self, data: u8) {
        self.cpu.core.borrow_mut().interrupts = data
    }
}

impl Addressable for CPUIORegisters {
    fn size(&self) -> usize {
        0xFF
    }

    fn const_read(&self, addr: u16) -> Result<u8, AddressingError> {
        match addr {
            0x0F => Ok(self.get_interrupts()),
            0x4C => Ok(self.get_key0()),
            0x4D => Ok(self.get_key1()),
            0xFF => Ok(self.get_interrupt_enable()),
            _ => Err(AddressingError::Unmapped),
        }
    }

    fn write(&self, addr: u16, data: u8) -> Result<(), AddressingError> {
        match addr {
            0x0F => {
                self.set_interrupts(data);
                Ok(())
            }
            0x4C => {
                self.set_key0(data);
                Ok(())
            }
            0x4D => {
                self.set_key1(data);
                Ok(())
            }
            0x50 => {
                self.set_boot_rom_lock();
                Ok(())
            }
            0xFF => {
                self.set_interrupt_enable(data);
                Ok(())
            }
            _ => Err(AddressingError::Unmapped),
        }
    }
}

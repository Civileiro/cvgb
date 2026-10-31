use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, Copy)]
enum PassCondition {
    Memory,
    SerialPassed,
}

impl PassCondition {
    pub fn finished(self, system: &game_boy::System, serial_output: &str) -> bool {
        match self {
            PassCondition::Memory => {
                system.get_context().debug_read_memory(0xA001) == Some(0xDE)
                    && system.get_context().debug_read_memory(0xA002) == Some(0xB0)
                    && system.get_context().debug_read_memory(0xA003) == Some(0x61)
                    && system.get_context().debug_read_memory(0xA000) != Some(0x80)
            }
            PassCondition::SerialPassed => {
                serial_output.contains("Passed") || serial_output.contains("Failed")
            }
        }
    }
    pub fn passed(self, system: &game_boy::System, serial_output: &str) -> bool {
        match self {
            PassCondition::Memory => system.get_context().debug_read_memory(0xA000).unwrap() == 0,
            PassCondition::SerialPassed => serial_output.contains("Passed"),
        }
    }
}

macro_rules!  testcases  {
    (
        $name:ident($path:expr, $cond:expr);
        $(
            $t_name:ident($t_path:expr, $t_cond:expr);
        )*
    ) => {
        mod $name {
            use super::blargg_test;
            use super::PassCondition::*;

            #[test]
            fn blargg_test_suite() {
                blargg_test($path,  $cond);
            }
        }
        $(
            testcases! {
                $t_name($t_path, $t_cond);
            }
        )*
    }
}

testcases! {
    // sound
    sound_01_registers("cgb_sound/rom_singles/01-registers", Memory);
    sound_02_len_ctr("cgb_sound/rom_singles/02-len ctr", Memory);
    sound_03_trigger("cgb_sound/rom_singles/03-trigger", Memory);
    sound_04_sweep("cgb_sound/rom_singles/04-sweep", Memory);
    sound_05_sweep_details("cgb_sound/rom_singles/05-sweep details", Memory);
    sound_06_overflow_on_trigger("cgb_sound/rom_singles/06-overflow on trigger", Memory);
    sound_07_len_sweep_period_sync("cgb_sound/rom_singles/07-len sweep period sync", Memory);
    sound_08_len_ctr_during_power("cgb_sound/rom_singles/08-len ctr during power", Memory);
    sound_09_wave_read_while_on("cgb_sound/rom_singles/09-wave read while on", Memory);
    sound_10_wave_trigger_while_on("cgb_sound/rom_singles/10-wave trigger while on", Memory);
    sound_11_regs_after_power("cgb_sound/rom_singles/11-regs after power", Memory);
    sound_12_wave("cgb_sound/rom_singles/12-wave", Memory);

    // instruction timing
    instr_timing("instr_timing/instr_timing", SerialPassed);

    // cpu_instrs
    cpu_instrs_01_special("cpu_instrs/individual/01-special", SerialPassed);
    cpu_instrs_02_interrupts("cpu_instrs/individual/02-interrupts", SerialPassed);
    cpu_instrs_03_op_sp_hl("cpu_instrs/individual/03-op sp,hl", SerialPassed);
    cpu_instrs_04_op_r_imm("cpu_instrs/individual/04-op r,imm", SerialPassed);
    cpu_instrs_05_op_rp("cpu_instrs/individual/05-op rp", SerialPassed);
    cpu_instrs_06_ld_r_r("cpu_instrs/individual/06-ld r,r", SerialPassed);
    cpu_instrs_07_jr_jp_call_ret_rst("cpu_instrs/individual/07-jr,jp,call,ret,rst", SerialPassed);
    cpu_instrs_08_misc_instrs("cpu_instrs/individual/08-misc instrs", SerialPassed);
    cpu_instrs_09_op_r_r("cpu_instrs/individual/09-op r,r", SerialPassed);
    cpu_instrs_10_bit_ops("cpu_instrs/individual/10-bit ops", SerialPassed);
    cpu_instrs_11_op_a_hl("cpu_instrs/individual/11-op a,(hl)", SerialPassed);

    // mem_timing
    mem_timing_01_read("mem_timing/individual/01-read_timing", SerialPassed);
    mem_timing_02_write("mem_timing/individual/02-write_timing", SerialPassed);
    mem_timing_03_modify("mem_timing/individual/03-modify_timing", SerialPassed);

    // mem_timing-2
    mem_timing2_01_read("mem_timing-2/rom_singles/01-read_timing", Memory);
    mem_timing2_02_write("mem_timing-2/rom_singles/02-write_timing", Memory);
    mem_timing2_03_modify("mem_timing-2/rom_singles/03-modify_timing", Memory);

    // oam_bug
    oam_bug_1_lcd_sync("oam_bug/rom_singles/1-lcd_sync", Memory);
    oam_bug_2_causes("oam_bug/rom_singles/2-causes", Memory);
    oam_bug_3_non_causes("oam_bug/rom_singles/3-non_causes", Memory);
    oam_bug_4_scanline_timing("oam_bug/rom_singles/4-scanline_timing", Memory);
    oam_bug_5_timing_bug("oam_bug/rom_singles/5-timing_bug", Memory);
    oam_bug_6_timing_no_bug("oam_bug/rom_singles/6-timing_no_bug", Memory);
    oam_bug_7_timing_effect("oam_bug/rom_singles/7-timing_effect", Memory);
    oam_bug_8_instr_effect("oam_bug/rom_singles/8-instr_effect", Memory);

    // interrupt_time, halt_bug
    interrupt_time("interrupt_time/interrupt_time", Memory);
    halt_bug("halt_bug", Memory);
}

fn blargg_test(name: &str, pass_condition: PassCondition) {
    let test_file = format!("../external/blargg-test-suite/{}.gb", name);
    let rom_path = PathBuf::from(&test_file);
    let rom = std::fs::read(rom_path).unwrap().into_boxed_slice();
    let mut cartridge = game_boy::Cartridge::from_rom(rom).unwrap();
    if cartridge.header.ram_size == 0x00 {
        cartridge.header.ram_size = 0x01;
    }
    let mut system = game_boy::System::new(cartridge, true);

    let max_duration = Duration::from_secs(30);

    let mut frame_duration = game_boy::SystemTime::from_seconds(max_duration.as_secs_f64());
    let mut serial_output = String::new();
    loop {
        let (_events, elapsed_time) = system.advance(frame_duration);
        serial_output.extend(
            system
                .get_serial_port_output()
                .into_iter()
                .map(|b| b as char),
        );
        if pass_condition.finished(&system, &serial_output) {
            break;
        }
        if elapsed_time >= frame_duration {
            println!(
                "{:x}",
                system.get_context().debug_read_memory(0xA000).unwrap()
            );
            println!(
                "{:x}",
                system.get_context().debug_read_memory(0xA001).unwrap()
            );
            println!(
                "{:x}",
                system.get_context().debug_read_memory(0xA002).unwrap()
            );
            println!(
                "{:x}",
                system.get_context().debug_read_memory(0xA003).unwrap()
            );
            panic!("Test did not finish\n{serial_output}");
        } else {
            frame_duration -= elapsed_time
        }
    }
    if pass_condition.passed(&system, &serial_output) {
        return;
    }
    if serial_output.is_empty() {
        for i in 0xA004.. {
            let data = system.get_context().debug_read_memory(i).unwrap();
            if data == 0 {
                break;
            }
            serial_output.push(data as char);
        }
    }
    panic!("Test failed\n{serial_output}");
}

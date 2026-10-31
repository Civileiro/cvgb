use std::{path::PathBuf, time::Duration};

macro_rules! testcases {
    (
        $name:ident($path:expr);
        $(
            $t_name:ident($t_path:expr);
        )*
    ) => {
        mod $name  {
            use super::mooneye_test;

            #[test]
            fn mooneye_test_suite() {
                mooneye_test($path);
            }
        }
        $(
            testcases! {
                $t_name($t_path);
            }
        )*
    }
}
testcases! {
    add_sp_e_timing("acceptance/add_sp_e_timing");
    call_cc_timing("acceptance/call_cc_timing");
    call_cc_timing2("acceptance/call_cc_timing2");
    call_timing("acceptance/call_timing");
    call_timing2("acceptance/call_timing2");
    div_timing("acceptance/div_timing");
    ei_sequence("acceptance/ei_sequence");
    ei_timing("acceptance/ei_timing");
    halt_ime0_ei("acceptance/halt_ime0_ei");
    halt_ime0_nointr_timing("acceptance/halt_ime0_nointr_timing");
    halt_ime1_timing("acceptance/halt_ime1_timing");
    if_ie_registers("acceptance/if_ie_registers");
    intr_timing("acceptance/intr_timing");
    jp_cc_timing("acceptance/jp_cc_timing");
    jp_timing("acceptance/jp_timing");
    ld_hl_sp_e_timing("acceptance/ld_hl_sp_e_timing");
    oam_dma_restart("acceptance/oam_dma_restart");
    oam_dma_start("acceptance/oam_dma_start");
    oam_dma_timing("acceptance/oam_dma_timing");
    pop_timing("acceptance/pop_timing");
    push_timing("acceptance/push_timing");
    rapid_di_ei("acceptance/rapid_di_ei");
    ret_timing("acceptance/ret_timing");
    reti_timing("acceptance/reti_timing");
    ret_cc_timing("acceptance/ret_cc_timing");
    reti_intr_timing("acceptance/reti_intr_timing");
    rst_timing("acceptance/rst_timing");
    bits_mem_oam("acceptance/bits/mem_oam");
    bits_reg_f("acceptance/bits/reg_f");
    instr_daa("acceptance/instr/daa");
    interrupts_ie_push("acceptance/interrupts/ie_push");
    oam_dma_basic("acceptance/oam_dma/basic");
    oam_dma_reg_read("acceptance/oam_dma/reg_read");
    ppu_intr_2_0_timing("acceptance/ppu/intr_2_0_timing");
    ppu_intr_2_mode0_timing("acceptance/ppu/intr_2_mode0_timing");
    ppu_stat_lyc_onoff("acceptance/ppu/stat_lyc_onoff");
    ppu_intr_2_mode0_timing_sprites("acceptance/ppu/intr_2_mode0_timing_sprites");
    ppu_intr_2_mode3_timing("acceptance/ppu/intr_2_mode3_timing");
    ppu_intr_2_oam_ok_timing("acceptance/ppu/intr_2_oam_ok_timing");
    ppu_stat_irq_blocking("acceptance/ppu/stat_irq_blocking");
    timer_div_write("acceptance/timer/div_write");
    timer_rapid_toggle("acceptance/timer/rapid_toggle");
    timer_tim00("acceptance/timer/tim00");
    timer_tim00_div_trigger("acceptance/timer/tim00_div_trigger");
    timer_tim01("acceptance/timer/tim01");
    timer_tim01_div_trigger("acceptance/timer/tim01_div_trigger");
    timer_tim10("acceptance/timer/tim10");
    timer_tim10_div_trigger("acceptance/timer/tim10_div_trigger");
    timer_tim11("acceptance/timer/tim11");
    timer_tim11_div_trigger("acceptance/timer/tim11_div_trigger");
    timer_tima_reload("acceptance/timer/tima_reload");
    timer_tima_write_reloading("acceptance/timer/tima_write_reloading");
    timer_tma_write_reloading("acceptance/timer/tma_write_reloading");
    mbc1_bits_ramg("emulator-only/mbc1/bits_ramg");
    mbc1_bits_bank1("emulator-only/mbc1/bits_bank1");
    mbc1_bits_bank2("emulator-only/mbc1/bits_bank2");
    mbc1_bits_mode("emulator-only/mbc1/bits_mode");
    mbc1_rom_512kb("emulator-only/mbc1/rom_512kb");
    mbc1_rom_1mb("emulator-only/mbc1/rom_1Mb");
    mbc1_rom_2mb("emulator-only/mbc1/rom_2Mb");
    mbc1_rom_4mb("emulator-only/mbc1/rom_4Mb");
    mbc1_rom_8mb("emulator-only/mbc1/rom_8Mb");
    mbc1_rom_16mb("emulator-only/mbc1/rom_16Mb");
    mbc1_ram_64kb("emulator-only/mbc1/ram_64kb");
    mbc1_ram_25kb("emulator-only/mbc1/ram_256kb");
    mbc1_multicart_rom_8mb("emulator-only/mbc1/multicart_rom_8Mb");
    mbc2_bits_ramg("emulator-only/mbc2/bits_ramg");
    mbc2_bits_romb("emulator-only/mbc2/bits_romb");
    mbc2_bits_unused("emulator-only/mbc2/bits_unused");
    mbc2_rom_512kb("emulator-only/mbc2/rom_512kb");
    mbc2_rom_1mb("emulator-only/mbc2/rom_1Mb");
    mbc2_rom_2mb("emulator-only/mbc2/rom_2Mb");
    mbc2_ram("emulator-only/mbc2/ram");
    mbc5_rom_512kb("emulator-only/mbc5/rom_512kb");
    mbc5_rom_1mb("emulator-only/mbc5/rom_1Mb");
    mbc5_rom_2mb("emulator-only/mbc5/rom_2Mb");
    mbc5_rom_4mb("emulator-only/mbc5/rom_4Mb");
    mbc5_rom_8mb("emulator-only/mbc5/rom_8Mb");
    mbc5_rom_16mb("emulator-only/mbc5/rom_16Mb");
    mbc5_rom_32mb("emulator-only/mbc5/rom_32Mb");
    mbc5_rom_64mb("emulator-only/mbc5/rom_64Mb");
    misc_boot_regs_cgb("misc/boot_regs-cgb");
    misc_bits_unused_hwio("misc/bits/unused_hwio-C");
}

fn mooneye_test(name: &str) {
    let test_file = format!("../external/mooneye-test-suite/build/{}.gb", name);
    let rom_path = PathBuf::from(&test_file);
    let rom = std::fs::read(rom_path).unwrap().into_boxed_slice();
    let cartridge = game_boy::Cartridge::from_rom(rom).unwrap();
    let mut system = game_boy::System::new(cartridge, true);

    let max_duration = Duration::from_secs(10);

    let mut frame_duration = game_boy::SystemTime::from_seconds(max_duration.as_secs_f64());
    let mut registers = None;
    let mut got_regs = false;
    loop {
        let (events, elapsed_time) = system.advance(frame_duration);
        if events.has_breakpoint() {
            if !got_regs {
                registers = Some(system.get_cpu().get_registers());
                got_regs = true
            } else {
                break;
            }
        }
        if elapsed_time >= frame_duration {
            break;
        } else {
            frame_duration -= elapsed_time
        }
    }
    match registers {
        None => panic!("Test did not finish"),
        Some(regs) => {
            if regs.a != 0
                || regs.b != 3
                || regs.c != 5
                || regs.d != 8
                || regs.e != 13
                || regs.h != 21
                || regs.l != 34
            {
                let serial_output = system.get_serial_port_output();
                panic!("Test failed (Serial = '{:?}')", serial_output);
            }
        }
    }
}

use std::{path::PathBuf, time::Duration};

macro_rules! testcases {
    (
        $name:ident($path:expr);
        $(
            $t_name:ident($t_path:expr);
        )*
    ) => {
        mod $name  {
            use super::age_test;

            #[test]
            fn age_test_suite() {
                age_test($path);
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
    ei_halt_dmgc_cgbbce("halt/ei-halt-dmgC-cgbBCE");
    halt_m0_interrupt_dmgc_cgbbce("halt/halt-m0-interrupt-dmgC-cgbBCE");
    halt_prefetch_dmgc_cgbbce("halt/halt-prefetch-dmgC-cgbBCE");
    lcd_align_ly_cgbbc("lcd-align-ly/lcd-align-ly-cgbBC");
    // lcd_align_ly_cgbe("lcd-align-ly/lcd-align-ly-cgbE");
    // ly_cgbe("ly/ly-cgbE");
    ly_dmgc_cgbbc("ly/ly-dmgC-cgbBC");
    ly_ncmbc("ly/ly-ncmBC");
    // ly_ncme("ly/ly-ncmE");
    // oam_read_cgbe("oam/oam-read-cgbE");
    oam_read_dmgc_cgbbc("oam/oam-read-dmgC-cgbBC");
    oam_read_ncmbc("oam/oam-read-ncmBC");
    // oam_read_ncme("oam/oam-read-ncmE");
    oam_write_cgbbce("oam/oam-write-cgbBCE");
    oam_write_ncmbce("oam/oam-write-ncmBCE");
    spsw_interrupts_cgbbc("speed-switch/caution/spsw-interrupts-cgbBC");
    spsw_interrupts_cgbe("speed-switch/caution/spsw-interrupts-cgbE");
    spsw_ch2_lc_delay_cgbbce("speed-switch/spsw-ch2-lc-delay-cgbBCE");
    spsw_div_cgbbce("speed-switch/spsw-div-cgbBCE");
    spsw_mode0_cgbbce("speed-switch/spsw-mode0-cgbBCE");
    spsw_stop_prefetch_cgbbce("speed-switch/spsw-stop-prefetch-cgbBCE");
    spsw_tima_cgbbc("speed-switch/spsw-tima-cgbBC");
    // spsw_tima_cgbe("speed-switch/spsw-tima-cgbE");
    stat_int_dmgc_cgbbce("stat-interrupt/stat-int-dmgC-cgbBCE");
    stat_int_ncmbce("stat-interrupt/stat-int-ncmBCE");
    // stat_mode_cgbe("stat-mode/stat-mode-cgbE");
    stat_mode_dmgc_cgbbc("stat-mode/stat-mode-dmgC-cgbBC");
    stat_mode_ds_cgbbce("stat-mode/stat-mode-ds-cgbBCE");
    stat_mode_ncmbc("stat-mode/stat-mode-ncmBC");
    // stat_mode_ncme("stat-mode/stat-mode-ncmE");
    stat_mode_sprites_dmgc_cgbbce("stat-mode-sprites/stat-mode-sprites-dmgC-cgbBCE");
    stat_mode_sprites_ds_cgbbce("stat-mode-sprites/stat-mode-sprites-ds-cgbBCE");
    stat_mode_window_cgbbce("stat-mode-window/stat-mode-window-cgbBCE");
    stat_mode_window_ds_cgbbce("stat-mode-window/stat-mode-window-ds-cgbBCE");
    stat_mode_window_ncmbce("stat-mode-window/stat-mode-window-ncmBCE");
    vram_read_cgbbce("vram/vram-read-cgbBCE");
    vram_read_ncmbce("vram/vram-read-ncmBCE");
}

fn age_test(name: &str) {
    let test_file = format!("../tests/roms/age-test-roms/{}.gb", name);
    let rom_path = PathBuf::from(&test_file);
    let rom = std::fs::read(rom_path).unwrap().into_boxed_slice();
    let cartridge = game_boy::Cartridge::from_rom(rom).unwrap();
    let mut system = game_boy::System::new(cartridge, true);

    let max_duration = Duration::from_secs(10);

    let mut frame_duration = game_boy::SystemTime::from_seconds(max_duration.as_secs_f64());
    let registers;
    loop {
        let (events, elapsed_time) = system.advance(frame_duration);
        if events.has_breakpoint() {
            registers = Some(system.get_cpu().get_registers());
            break;
        }
        if elapsed_time >= frame_duration {
            panic!("Test did not finish");
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
                panic!("Test failed");
            }
        }
    }
}

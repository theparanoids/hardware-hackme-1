// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

extern crate cortex_m;
extern crate crc;
extern crate stm32f405;

use core;
use {PuzzlesStateSmol, PuzzlesStateLorge};

#[inline(never)]
#[link_section = ".data.flashprog_routines"]
pub fn flash_option_lock() {
    unsafe {
        // We _must_ use these raw operations. Using rtfm::atomic
        // _can_ cause codegen to place the "real code" back into
        // main flash because the "real code" is now a separate
        // closure.
        cortex_m::interrupt::disable();

        let flash = &*stm32f405::FLASH::ptr();

        // Unlock
        flash.optkeyr.write(|w| w.bits(0x08192A3B));
        flash.optkeyr.write(|w| w.bits(0x4C5D6E7F));

        while flash.sr.read().bsy().bit() {}

        // Value
        flash.optcr.write(|w| w.bits(0x0FFF00EC));
        // Start
        flash.optcr.write(|w| w.bits(0x0FFF00EE));

        // Wait completion
        while flash.sr.read().bsy().bit() {}

        // Relock
        flash.optcr.write(|w| w.bits(0x0FFF00ED));

        cortex_m::interrupt::enable();
    }
}

#[inline(never)]
#[link_section = ".data.flashprog_routines"]
pub fn flash_option_unlock() {
    unsafe {
        cortex_m::interrupt::disable();

        let flash = &*stm32f405::FLASH::ptr();

        // Unlock
        flash.optkeyr.write(|w| w.bits(0x08192A3B));
        flash.optkeyr.write(|w| w.bits(0x4C5D6E7F));

        while flash.sr.read().bsy().bit() {}

        // Value
        flash.optcr.write(|w| w.bits(0x0FFFAAEC));
        // Start
        flash.optcr.write(|w| w.bits(0x0FFFAAEE));

        // Wait completion
        while flash.sr.read().bsy().bit() {}

        // Relock
        flash.optcr.write(|w| w.bits(0x0FFFAAED));

        // There is nothing we can do now since the flash is now blank.
        // We can spin and blink the LEDs.
        let gpiob =  &*stm32f405::GPIOB::ptr();
        let tim7 =  &*stm32f405::TIM7::ptr();
        loop {
            gpiob.bsrr.write(|w| w
                .bs6().set()
                .bs7().set()
                .bs8().set()
                .bs9().set());

            for _ in 0..500 {
                while !tim7.sr.read().uif().bit() {}
                tim7.sr.modify(|_, w| w.uif().clear());
            }

            gpiob.bsrr.write(|w| w
                .br6().reset()
                .br7().reset()
                .br8().reset()
                .br9().reset());

            for _ in 0..500 {
                while !tim7.sr.read().uif().bit() {}
                tim7.sr.modify(|_, w| w.uif().clear());
            }
        }
    }
}

#[inline(never)]
#[link_section = ".data.flashprog_routines"]
pub fn flash_save_settings(state: &mut PuzzlesStateSmol, keeping_large_state: bool) {
    unsafe {
        cortex_m::interrupt::disable();

        assert!(core::mem::size_of::<PuzzlesStateSmol>() % 4 == 0);
        assert!(core::mem::size_of::<PuzzlesStateLorge>() % 4 == 0);
        assert!(core::mem::size_of::<PuzzlesStateSmol>() < 128*1024);
        assert!(core::mem::size_of::<PuzzlesStateLorge>() < 128*1024);

        // Bandaid: bits for cleared levels are inverted to prevent reset glitch
        state.cleared_levels ^= 0xFFFFFFFF;
        let buf_smol = core::slice::from_raw_parts(state as *const PuzzlesStateSmol as *const u32,
            core::mem::size_of::<PuzzlesStateSmol>() / 4);

        let buf_smol_u8 = core::slice::from_raw_parts(state as *const PuzzlesStateSmol as *const u8,
            core::mem::size_of::<PuzzlesStateSmol>());
        let buf_smol_crc = crc::crc32::checksum_ieee(buf_smol_u8);

        let flash = &*stm32f405::FLASH::ptr();

        // Turn caches off
        flash.acr.modify(|_, w| w
            .dcen().bit(false)
            .icen().bit(false));

        // Unlock
        flash.keyr.write(|w| w.bits(0x45670123));
        flash.keyr.write(|w| w.bits(0xCDEF89AB));

        while flash.sr.read().bsy().bit() {}

        if !keeping_large_state {
            // Erase sector 10
            flash.cr.write(|w| w.bits(0x00000252));
            flash.cr.write(|w| w.bits(0x00010252));

            // Wait completion
            while flash.sr.read().bsy().bit() {}
        }

        // Erase sector 11
        flash.cr.write(|w| w.bits(0x0000025A));
        flash.cr.write(|w| w.bits(0x0001025A));

        // Wait completion
        while flash.sr.read().bsy().bit() {}

        // Program mode
        flash.cr.write(|w| w.bits(0x00000201));
        let flash_sect_base = 0x080E0000 as *mut u32;
        for (i, &x) in buf_smol.iter().enumerate() {
            core::ptr::write_volatile(flash_sect_base.offset(i as isize), x);
            cortex_m::asm::dsb();
            while flash.sr.read().bsy().bit() {}
        }
        core::ptr::write_volatile(flash_sect_base.offset(buf_smol.len() as isize), buf_smol_crc);
        cortex_m::asm::dsb();
        while flash.sr.read().bsy().bit() {}

        if !keeping_large_state {
            // assume that it has already been loaded to 0x20008000
            let buf_lorge = core::slice::from_raw_parts(
                0x20008000 as *const u32,
                core::mem::size_of::<PuzzlesStateLorge>() / 4);
            let flash_sect_base = 0x080C0000 as *mut u32;
            for (i, &x) in buf_lorge.iter().enumerate() {
                core::ptr::write_volatile(flash_sect_base.offset(i as isize), x);
                cortex_m::asm::dsb();
                while flash.sr.read().bsy().bit() {}
            }
        }

        // Relock
        flash.cr.write(|w| w.bits(0x80000000));

        // Reset caches
        flash.acr.modify(|_, w| w
            .dcrst().bit(true)
            .icrst().bit(true));
        flash.acr.modify(|_, w| w
            .dcrst().bit(false)
            .icrst().bit(false));
        // Turn caches back on
        flash.acr.modify(|_, w| w
            .dcen().bit(true)
            .icen().bit(true));

        // Bandaid: bits for cleared levels are inverted to prevent reset glitch
        state.cleared_levels ^= 0xFFFFFFFF;

        cortex_m::interrupt::enable();
    }
}

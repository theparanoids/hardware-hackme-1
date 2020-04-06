// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use core::sync::atomic::AtomicU32;

use jtag::*;
use ::msleep;

const JTAG_OPC_IDCODE: u8   = 0b00000001;
const JTAG_OPC_BYPASS: u8   = 0b11111111;
const ISC_DISABLE: u8       = 0b11000000;
const ISC_ENABLE: u8        = 0b11101000;
const ISC_PROGRAM: u8       = 0b11101010;
const ISC_ERASE: u8         = 0b11101101;
const ISC_READ: u8          = 0b11101110;
const ISC_INIT: u8          = 0b11110000;

pub fn cpld_get_idcode(gpio: &::stm32f405::gpiob::RegisterBlock) -> u32 {
    test_logic_reset(gpio);
    shift_ir_from_tlr(gpio);

    shift_u8(gpio, JTAG_OPC_IDCODE, true);

    shift_dr_from_exit1_ir_dr(gpio);

    let idcode = shift_u32(gpio, 0, true);

    tlr_from_exit1_ir_dr(gpio);

    idcode
}

fn cpld_init_pulsing(gpio: &::stm32f405::gpiob::RegisterBlock, msleep_ctr: &'static AtomicU32) {
    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_INIT, true);
    set_tms(gpio, true);
    pulse_tck(gpio);         // Update-IR
    pulse_tck(gpio);         // Select DR-Scan
    set_tms(gpio, false);
    pulse_tck(gpio);         // Capture-DR
    set_tms(gpio, true);
    pulse_tck(gpio);         // Exit1-DR
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);
}

fn cpld_isc_disable_and_bypass(gpio: &::stm32f405::gpiob::RegisterBlock) {
    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_DISABLE, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    shift_ir_from_rti(gpio);
    shift_u8(gpio, JTAG_OPC_BYPASS, true);
    tlr_from_exit1_ir_dr(gpio);
    pulse_tck(gpio);
}

pub fn cpld_erase(gpio: &::stm32f405::gpiob::RegisterBlock, msleep_ctr: &'static AtomicU32) {
    test_logic_reset(gpio);

    shift_ir_from_tlr(gpio);
    shift_u8(gpio, ISC_ENABLE, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);

    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_ERASE, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 100);

    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_INIT, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);

    cpld_init_pulsing(gpio, msleep_ctr);

    cpld_isc_disable_and_bypass(gpio);
}

pub fn cpld_read_eeprom<F>(gpio: &::stm32f405::gpiob::RegisterBlock, msleep_ctr: &'static AtomicU32,
    mut callback: F) where F: FnMut(u32, u32, bool) {

    test_logic_reset(gpio);

    shift_ir_from_tlr(gpio);
    shift_u8(gpio, JTAG_OPC_BYPASS, false);
    shift_u8(gpio, ISC_ENABLE, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);

    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_READ, true);
    shift_dr_from_exit1_ir_dr(gpio);

    for addr in 0..50 {
        let addr_gray: u32 = addr ^ (addr >> 1);

        shift_bit(gpio, (addr_gray & (1 << 5)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 4)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 3)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 2)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 1)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 0)) != 0, true);
        run_test_idle_from_exit1_ir_dr(gpio);
        msleep(msleep_ctr, 1);
        shift_dr_from_rti(gpio);

        for i in 0..260 {
            callback(addr, i, shift_bit(gpio, true, addr == 49 && i == 259));
        }
    }

    // In Exit1-DR now
    shift_ir_from_exit1_ir_dr(gpio);
    shift_u8(gpio, ISC_INIT, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);

    cpld_init_pulsing(gpio, msleep_ctr);

    cpld_isc_disable_and_bypass(gpio);
}

pub fn cpld_write_eeprom<F>(gpio: &::stm32f405::gpiob::RegisterBlock, msleep_ctr: &'static AtomicU32,
    mut callback: F) where F: FnMut(u32, u32) -> bool {

    test_logic_reset(gpio);

    shift_ir_from_tlr(gpio);
    shift_u8(gpio, ISC_ENABLE, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);

    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_PROGRAM, true);
    shift_dr_from_exit1_ir_dr(gpio);

    for addr in 0..49 {
        let addr_gray: u32 = addr ^ (addr >> 1);

        for i in 0..260 {
            let bit = callback(addr, 259 - i);
            shift_bit(gpio, bit, false);
        }

        shift_bit(gpio, (addr_gray & (1 << 5)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 4)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 3)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 2)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 1)) != 0, false);
        shift_bit(gpio, (addr_gray & (1 << 0)) != 0, true);
        run_test_idle_from_exit1_ir_dr(gpio);
        msleep(msleep_ctr, 10);

        if addr != 48 {
            shift_dr_from_rti(gpio);
        }
    }

    // In Run-Test/Idle now
    shift_ir_from_rti(gpio);
    shift_u8(gpio, ISC_INIT, true);
    run_test_idle_from_exit1_ir_dr(gpio);
    msleep(msleep_ctr, 1);

    cpld_init_pulsing(gpio, msleep_ctr);

    cpld_isc_disable_and_bypass(gpio);
}

// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

pub fn set_tms(gpio: &::stm32f405::gpiob::RegisterBlock, bit: bool) {
    if bit {
        gpio.bsrr.write(|w| w.bs12().set());
    } else {
        gpio.bsrr.write(|w| w.br12().reset());
    }
}

pub fn get_tdo(gpio: &::stm32f405::gpiob::RegisterBlock) -> bool {
    gpio.idr.read().idr14().bit()
}

pub fn set_tdi(gpio: &::stm32f405::gpiob::RegisterBlock, bit: bool) {
    if bit {
        gpio.bsrr.write(|w| w.bs15().set());
    } else {
        gpio.bsrr.write(|w| w.br15().reset());
    }
}

pub fn pulse_tck(gpio: &::stm32f405::gpiob::RegisterBlock) {
    gpio.bsrr.write(|w| w.bs13().set());
    unsafe {
        asm!("nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop"
            :
            :
            :
            : "volatile");
    }
    gpio.bsrr.write(|w| w.br13().reset());
    unsafe {
        asm!("nop
            nop
            nop
            nop
            nop
            nop
            nop
            nop"
            :
            :
            :
            : "volatile");
    }
}

pub fn test_logic_reset(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tdi(gpio, false);
    set_tms(gpio, true);
    pulse_tck(gpio);
    pulse_tck(gpio);
    pulse_tck(gpio);
    pulse_tck(gpio);
    pulse_tck(gpio);
}

pub fn shift_ir_from_tlr(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, false);
    pulse_tck(gpio);         // Run-Test/Idle
    set_tms(gpio, true);
    pulse_tck(gpio);         // Select DR-Scan
    pulse_tck(gpio);         // Select IR-Scan
    set_tms(gpio, false);
    pulse_tck(gpio);         // Capture-IR
    pulse_tck(gpio);         // Shift-IR
}

pub fn shift_ir_from_rti(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, true);
    pulse_tck(gpio);         // Select DR-Scan
    pulse_tck(gpio);         // Select IR-Scan
    set_tms(gpio, false);
    pulse_tck(gpio);         // Capture-IR
    pulse_tck(gpio);         // Shift-IR
}

pub fn shift_dr_from_rti(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, true);
    pulse_tck(gpio);         // Select DR-Scan
    set_tms(gpio, false);
    pulse_tck(gpio);         // Capture-DR
    pulse_tck(gpio);         // Shift-DR
}

pub fn shift_ir_from_exit1_ir_dr(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, true);
    pulse_tck(gpio);         // Update-IR
    pulse_tck(gpio);         // Select DR-Scan
    pulse_tck(gpio);         // Select IR-Scan
    set_tms(gpio, false);
    pulse_tck(gpio);         // Capture-IR
    pulse_tck(gpio);         // Shift-IR
}

pub fn shift_dr_from_exit1_ir_dr(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, true);
    pulse_tck(gpio);         // Update-IR
    pulse_tck(gpio);         // Select DR-Scan
    set_tms(gpio, false);
    pulse_tck(gpio);         // Capture-DR
    pulse_tck(gpio);         // Shift-DR
}

pub fn tlr_from_exit1_ir_dr(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, true);
    pulse_tck(gpio);         // Update-IR/DR
    pulse_tck(gpio);         // Select DR-Scan
    pulse_tck(gpio);         // Select IR-Scan
    pulse_tck(gpio);         // TLR
}

pub fn run_test_idle_from_exit1_ir_dr(gpio: &::stm32f405::gpiob::RegisterBlock) {
    set_tms(gpio, true);
    pulse_tck(gpio);         // Update-IR/DR
    set_tms(gpio, false);
    pulse_tck(gpio);         // Run-Test/Idle
}

pub fn shift_bit(gpio: &::stm32f405::gpiob::RegisterBlock, val: bool, exit: bool) -> bool {
    let result = get_tdo(gpio);
    set_tdi(gpio, val);

    if exit {
        set_tms(gpio, true);    // Exit1
    }

    pulse_tck(gpio);

    if exit {
        set_tms(gpio, false);   // Other code assumes TMS=0
    }

    result
}

pub fn shift_u8(gpio: &::stm32f405::gpiob::RegisterBlock, val: u8, exit: bool) -> u8 {
    let mut result = 0;

    for i in 0..8 {
        if get_tdo(gpio) {
            result |= 1 << i;
        }

        set_tdi(gpio, (val & (1 << i)) != 0);

        if i == 7 && exit {
            set_tms(gpio, true);    // Exit1
        }

        pulse_tck(gpio);
    }

    if exit {
        set_tms(gpio, false);   // Other code assumes TMS=0
    }

    result
}

pub fn shift_u32(gpio: &::stm32f405::gpiob::RegisterBlock, val: u32, exit: bool) -> u32 {
    let mut result = 0;

    for i in 0..32 {
        if get_tdo(gpio) {
            result |= 1 << i;
        }

        set_tdi(gpio, (val & (1 << i)) != 0);

        if i == 31 && exit {
            set_tms(gpio, true);    // Exit1
        }

        pulse_tck(gpio);
    }

    if exit {
        set_tms(gpio, false);   // Other code assumes TMS=0
    }

    result
}

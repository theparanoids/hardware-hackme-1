// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

#![feature(alloc)]
#![feature(global_allocator)]
#![feature(core_intrinsics)]
#![feature(lang_items)]
#![feature(proc_macro)]
#![feature(integer_atomics)]
#![feature(const_fn)]
#![feature(naked_functions)]
#![feature(asm)]
#![feature(reverse_bits)]
#![feature(used)]
#![no_std]

extern crate aes_nostd;
extern crate alloc_cortex_m;
extern crate alloc;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m_semihosting;
extern crate crc;
extern crate stm32f405;
extern crate heapless;
extern crate cryptastic;
extern crate sha2;

use alloc::string::ToString;
use alloc_cortex_m::CortexMHeap;
use rtfm::{app, Threshold, Resource};

use core::sync::atomic::{AtomicU32, Ordering};

use heapless::ring_buffer::{RingBuffer, Producer, Consumer};

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

extern "C" {
    static mut _sheap: u32;
    static mut _eheap: u32;
}

#[allow(dead_code)]
const COREFREQ: u32 = 168_000_000;
#[allow(dead_code)]
const AHBFREQ: u32 = 168_000_000;
#[allow(dead_code)]
const APB2FREQ: u32 = 84_000_000;
#[allow(dead_code)]
const APB1FREQ: u32 = 42_000_000;

#[macro_use]
mod cli;
#[macro_use]
use cli::*;
mod crypto;
mod flash;
use flash::*;
mod jtag;
mod cpld;
use cpld::*;
mod sfx;
use sfx::{FOUR_NOTE_COMPLETE_SFX, speaker_set_freq, play_song};
mod except_and_svc;
pub use except_and_svc::{SVCALL, DEFAULT_HANDLER, rust_begin_unwind, SVCallCTX};
mod basic_puzzles;
use basic_puzzles::*;
mod prng_puzzles;
use prng_puzzles::*;
mod crypto_puzzles;
use crypto_puzzles::*;
mod cpld_puzzles;
use cpld_puzzles::*;

#[repr(C)]
pub struct PuzzlesStateSmol {
    pub last_cleared_level: i32,
    pub cleared_levels: u32,
    pub level0_pin: [u8; 4],
    pub level1_pin: [u8; 4],
    pub level2_pin: [u8; 4],
    pub level2_tries_left: u32,
    pub enable_sound: bool,
}

#[repr(C)]
pub struct PuzzlesStateLorge {
    pub level7_modulus: [u32; 64],
    pub level7_r: [u32; 64],
    pub level7_rr: [u32; 64],
    pub level7_example: [u8; 384],
    pub level8_pubk_x: [u32; 8],
    pub level8_pubk_y: [u32; 8],
    pub level8_pubk_plus_g_x: [u32; 8],
    pub level8_pubk_plus_g_y: [u32; 8],
    pub level8_example1: [u8; 192],
    pub level8_example2: [u8; 192],
    pub level9_code: [u8; 8192],
    pub level9_xor: [u8; 8192],
    pub level10_key_orig: [u8; 16],
    pub level10_code: [u8; 8192],
    pub level4_seed: [u32; 10],
    pub cpld_16bit_answer: [u8; 4],
    pub cpld_64bit_answer: [u8; 8],
    pub cpld_center_hidden: [u8; 8],
    pub cpld_pterm_hidden: [u8; 70],
    pub flags: [u8; 32*16],
}

impl PuzzlesStateSmol {
    pub const fn new() -> Self {
        Self {
            last_cleared_level: -1,
            cleared_levels: 0,
            level0_pin: [5, 6, 7, 9],
            level1_pin: [1, 2, 3, 5],
            level2_pin: [6, 6, 6, 7],
            level2_tries_left: 3,
            enable_sound: true,
        }
    }
}

// These are needed for hiding secret state for the
// PRNG levels to defend against dumping RAM after a
// debugger reset.
const BKPSRAM_ADDR_0: u32 = 0x40024000;
const BKPSRAM_ADDR_3: u32 = 0x40024F00;

mod hax {
    extern "C" {
        pub fn main(argc: isize, argv: *const *const u8) -> isize;
    }
}
#[used]
#[no_mangle]
pub extern "C" fn hijack_main(argc: isize, argv: *const *const u8) -> isize {
    unsafe {
        // Enable access to the backup SRAM which we will use for
        // "sensitive" state
        let rcc = &*stm32f405::RCC::ptr();
        let pwr = &*stm32f405::PWR::ptr();

        rcc.apb1enr.modify(|_, w| w.pwren().enabled());
        pwr.cr.modify(|_, w| w.dbp().bit(true));
        rcc.ahb1enr.modify(|_, w| w.bkpsramen().enabled());

        stack_do_switcheroo();

        ::hax::main(argc, argv)
    }
}

#[inline(never)]
#[naked]
unsafe extern "C" fn stack_do_switcheroo() {
    asm!("

        dsb
        isb

        movw r3, #0x4000
        movt r3, #0x1000
        mov r2, sp
        rsb r2, r2, r3
        movw r1, #0x4f00
        movt r1, #0x4002
        subs r1, r1, r2
        mov r2, sp
        mov sp, r1
        1:
        ldmia r2!, {r0}
        stmia r1!, {r0}
        cmp r2, r3
        bne 1b

        dsb
        isb

        bx lr
        "
        :
        :
        :
        : "volatile");
}

app! {
    device: stm32f405,

    resources: {
        static GPIOB: stm32f405::GPIOB;
        static TIM7_OBJ: stm32f405::TIM7;
        static USER_UART: stm32f405::USART2;
        static CPLD_UART: stm32f405::USART6;
        static SPEAKER_TIMER: stm32f405::TIM2;

        // Millisecond counter
        static MS_COUNTER: AtomicU32 = AtomicU32::new(0xFFFFFF00);
        static TIM7_MS_COUNTER: &'static AtomicU32;
        static IDLE_MS_COUNTER: &'static AtomicU32;

        // Current input line
        static LINEEDIT_STATE: LineEditState = LineEditState::new();

        static CMDLINE_RX_RB: RingBuffer<&'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]> = RingBuffer::new();
        static CMDLINE_RX_P: Producer<'static, &'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]>;
        static CMDLINE_RX_C: Consumer<'static, &'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]>;

        static CMDLINE_RET_RB: RingBuffer<&'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]> = RingBuffer::new();
        static CMDLINE_RET_P: Producer<'static, &'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]>;
        static CMDLINE_RET_C: Consumer<'static, &'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]>;

        static PUZZLES_STATE: &'static mut PuzzlesStateSmol;
        static PUZZLES_STATE_LORGE: &'static PuzzlesStateLorge;

        static RNG: stm32f405::RNG;
        static RNG_LAST_VAL: u32;

        static DEBUG_MODE: bool;
        static SAVE_DATA_VALID: bool;

        // Non-sensitive state that is too big to fit in stack
        static WORK_BIGNUM_2048: cryptastic::Bignum2048 = cryptastic::Bignum2048([0; 64]);
        static WORK_BIGNUM_2048_2: cryptastic::Bignum2048 = cryptastic::Bignum2048([0; 64]);
        static WORK_BIGNUM_2048_3: cryptastic::Bignum2048 = cryptastic::Bignum2048([0; 64]);
        static WORK_BIGNUM_2048_4: cryptastic::Bignum2048 = cryptastic::Bignum2048([0; 64]);
        static WORK_BIGNUM_2048O: cryptastic::Bignum2048Oversized = cryptastic::Bignum2048Oversized([0; 65]);
        static WORK_BIGNUM_4096O: cryptastic::Bignum4096Oversized = cryptastic::Bignum4096Oversized([0; 129]);

        static WORK_BIGNUM_256_0: cryptastic::Bignum256 = cryptastic::Bignum256([0; 8]);
        static WORK_BIGNUM_256_1: cryptastic::Bignum256 = cryptastic::Bignum256([0; 8]);
        static WORK_BIGNUM_256_2: cryptastic::Bignum256 = cryptastic::Bignum256([0; 8]);

        static LEVEL7_MODULUS: cryptastic::Bignum2048;
        static LEVEL7_R: cryptastic::Bignum2048;
        static LEVEL7_RR: cryptastic::Bignum2048;

        static LEVEL8_PUBK: cryptastic::ECPointAffine256;
        static LEVEL8_PUBK_PLUS_G: cryptastic::ECPointAffine256;

        static LEVEL78_UPLOAD_BUF: [u8; 384] = [0u8; 384];

        static WORK_CMDLINE_BUF_0: CmdlineBuf = CmdlineBuf { buf: [0u8; 256], len: 0 };
        static WORK_CMDLINE_BUF_1: CmdlineBuf = CmdlineBuf { buf: [0u8; 256], len: 0 };
    },

    init: {
        resources: [MS_COUNTER, CMDLINE_RX_RB, CMDLINE_RET_RB, WORK_CMDLINE_BUF_0, WORK_CMDLINE_BUF_1],
    },

    idle: {
        resources: [GPIOB, USER_UART, IDLE_MS_COUNTER, CMDLINE_RX_C, CMDLINE_RET_P,
            PUZZLES_STATE, PUZZLES_STATE_LORGE, RNG, RNG_LAST_VAL, DEBUG_MODE,
            SAVE_DATA_VALID, CPLD_UART, SPEAKER_TIMER,

            WORK_BIGNUM_2048, WORK_BIGNUM_2048O, WORK_BIGNUM_4096O,
            WORK_BIGNUM_2048_2, WORK_BIGNUM_2048_3, WORK_BIGNUM_2048_4,

            WORK_BIGNUM_256_0, WORK_BIGNUM_256_1, WORK_BIGNUM_256_2,

            LEVEL7_MODULUS, LEVEL7_R, LEVEL7_RR, LEVEL78_UPLOAD_BUF,
            LEVEL8_PUBK, LEVEL8_PUBK_PLUS_G],
    },

    tasks: {
        USART2: {
            path: uart_rx_lineedit,

            resources: [USER_UART, LINEEDIT_STATE, CMDLINE_RX_P, CMDLINE_RET_C],
        },

        TIM7: {
            path: millisecond_intr,

            resources: [TIM7_OBJ, TIM7_MS_COUNTER],
            priority: 2,
        }
    }
}

static mut CALL_PAYLOAD_STASHED_CTX: *mut SVCallCTX = core::ptr::null_mut();
static mut LEVEL9_CLEAR_FLAG: bool = false;
static mut LEVEL10_CLEAR_FLAG: bool = false;

fn init(p: init::Peripherals, r: init::Resources) -> init::LateResources {
    // Initialize the allocator
    let start = unsafe { &mut _sheap as *mut u32 as usize };
    let end = unsafe { &mut _eheap as *mut u32 as usize };
    unsafe { ALLOCATOR.init(start, end - start) }

    // Set up clocks
    let rcc = p.device.RCC;

    // Turn clock-related interrupts off and clear them
    rcc.cir.reset();
    rcc.cir.modify(|_, w| w
        .cssc().clear()
        .plli2srdyc().clear()
        .pllrdyc().clear()
        .hserdyc().clear()
        .hsirdyc().clear()
        .lserdyc().clear()
        .lsirdyc().clear());

    // Turn on enternal clock
    rcc.cr.modify(|_, w| w.hseon().bit(false));
    rcc.cr.modify(|_, w| w.hsebyp().bit(false));
    rcc.cr.modify(|_, w| w.hseon().bit(true));
    while !rcc.cr.read().hserdy().bit() {}

    // Turn on PLL (turn off, configure, turn on)
    rcc.cr.modify(|_, w| w.pllon().bit(false));
    // Turn off spread-spectrum
    rcc.sscgr.reset();
    // Need to keep reserved values
    // We are configuring a 336 MHz VCO from an 8 MHz input
    // in order to output a system clock of 168 MHz and a USB/SDIO clock of
    // 48 MHz.
    rcc.pllcfgr.modify(|_, w| w
        .pllm().bits(4)             // 8 MHz to 2 MHz
        .plln().bits(168)           // 2 MHz to 336 MHz
        .pllp()._2()                // 336 MHz to 168 MHz
        .pllsrc().hse()             // Use crystal
        .pllq().bits(7));           // 336 MHz to 48 MHz
    rcc.cr.modify(|_, w| w.pllon().bit(true));
    while !rcc.cr.read().pllrdy().bit() {}

    // Enable flash caching and prefetching
    // According to the table we need 5 waitstates
    // We also need to do this _before_ switching clock sources.
    p.device.FLASH.acr.write(|w| w
        .dcen().bit(true)
        .icen().bit(true)
        .prften().bit(true)
        .latency().bits(5));

    // Set the prescalers so that AHB is 168 MHz, APB2 is 84 MHz, APB1 is 42 MHz
    rcc.cfgr.write(|w| w
        .ppre2()._2()
        .ppre1()._4()
        .hpre()._1());
    rcc.cfgr.modify(|_, w| w.sw().pll());
    while !rcc.cfgr.read().sws().is_pll() {}

    // Turn off HSI and CSS, we don't need them
    rcc.cr.modify(|_, w| w.hsion().bit(false).csson().bit(false));

    // Power up the relevant peripherals
    rcc.ahb1enr.modify(|_, w| w
        .gpioaen().enabled()
        .gpioben().enabled()
        .gpiocen().enabled());
    rcc.ahb2enr.modify(|_, w| w
        .rngen().enabled());
    rcc.apb1enr.modify(|_, w| w
        .tim2en().enabled()
        .tim7en().enabled()
        .usart2en().enabled());
    rcc.apb2enr.modify(|_, w| w
        .usart6en().enabled());

    // Ensure VTOR points to flash
    unsafe { p.core.SCB.vtor.write(0x08000000); }

    // If we are locked, then disable the JTAG/SWD interface before sensitive data
    // ever ends up in RAM
    let gpioa = p.device.GPIOA;
    let gpiob = p.device.GPIOB;
    let optionbytes = p.device.FLASH.optcr.read().bits();
    let mut debugmode = true;
    if optionbytes & 0xFF00 != 0xAA00 {
        gpioa.moder.modify(|_, w| w
            .moder13().input()
            .moder14().input()
            .moder15().input());
        gpiob.moder.modify(|_, w| w
            .moder3().input()
            .moder4().input());
        debugmode = false;
    }

    // Set up USART2 for 115200 8N1
    let usart2 = p.device.USART2;
    usart2.brr.write(|w| w.div_mantissa().bits(22).div_fraction().bits(13));
    usart2.cr1.write(|w| w.ue().bit(true).te().bit(true).re().bit(true));
    usart2.cr2.write(|w| w.stop()._1());
    usart2.cr3.write(|w| w);

    // Set up USART2 on PA2/3
    gpioa.afrl.modify(|_, w| w.afrl2().af7().afrl3().af7());
    gpioa.moder.modify(|_, w| w.moder2().alternate().moder3().alternate());

    // Enable the USART2 RXNE interrupt
    usart2.cr1.modify(|_, w| w.rxneie().bit(true));

    // Set up USART6 for 1M 8N1
    let usart6 = p.device.USART6;
    usart6.brr.write(|w| w.div_mantissa().bits(5).div_fraction().bits(4));
    usart6.cr1.write(|w| w.ue().bit(true).te().bit(true).re().bit(true));
    usart6.cr2.write(|w| w.stop()._1());
    usart6.cr3.write(|w| w);

    // Set up USART6 on PC6/7
    let gpioc = p.device.GPIOC;
    gpioc.afrl.modify(|_, w| w.afrl6().af8().afrl7().af8());
    gpioc.moder.modify(|_, w| w.moder6().alternate().moder7().alternate());

    // Set up the CPLD clock (8 MHz, matching our HSE)
    rcc.cfgr.modify(|_, w| w
        .mco1pre()._0()
        .mco1().hse());
    // This pin needs to be medium speed
    unsafe {
        gpioa.ospeedr.modify(|_, w| w.ospeedr8().bits(0b01));
    }
    gpioa.afrh.modify(|_, w| w.afrh8().af0());
    gpioa.moder.modify(|_, w| w.moder8().alternate());

    // Configure the pins PB6-9 as output pins for LEDs
    gpiob.moder.modify(|_, w| w
        .moder6().output()
        .moder7().output()
        .moder8().output()
        .moder9().output());

    // Configure the pins PB12-15 for CPLD JTAG
    // PB12 - TMS
    // PB13 - TCK
    // PB14 - TDO
    // PB15 - TDI
    gpiob.moder.modify(|_, w| w
        .moder12().output()
        .moder13().output()
        .moder14().input()
        .moder15().output());
    // These pins need to be medium speed
    unsafe {
        gpiob.ospeedr.modify(|_, w| w
            .ospeedr12().bits(0b01)
            .ospeedr13().bits(0b01)
            .ospeedr15().bits(0b01));
    }
    // Ensure TCK is low
    gpiob.bsrr.write(|w| w.br13().reset());

    // Set up TIM7 for 1 ms interrupts
    let tim7 = p.device.TIM7;
    tim7.psc.write(|w| w.psc().bits(1000 - 1));
    // NOTE NOTE NOTE: If APB prescaler not 1, timer frequency is doubled.
    tim7.arr.write(|w| w.arr().bits((2 * APB1FREQ / 1_000_000 - 1) as u16));
    tim7.cr1.write(|w| w.opm().continuous());
    tim7.dier.write(|w| w.uie().bit(true));
    tim7.cr1.modify(|_, w| w.cen().enabled());

    // Set up the speaker
    let tim2 = p.device.TIM2;
    tim2.psc.write(|w| w.psc().bits(0));        // Full 168??? MHz
    tim2.cr1.write(|w| w
        .arpe().bit(true));                     // Must enable preload, everything else is correct
                                                // (edge aligned, up counter, not one-pulse)
                                                // but we don't enable yet
    unsafe {
        tim2.ccmr2_output.write(|w| w
            .oc4m().bits(0b110)
            .oc4pe().bit(true));                // Output
    }
    tim2.ccer.write(|w| w
        .cc4e().bit(true));                     // Enabled, not inverted
    speaker_set_freq(&tim2, 0.);
    tim2.egr.write(|w| w.ug().bit(true));       // Kick UG
    tim2.cr1.modify(|_, w| w.cen().enabled());  // Enable

    // Set up the speaker IO pin
    gpiob.afrh.modify(|_, w| w.afrh11().af1());
    gpiob.moder.modify(|_, w| w.moder11().alternate());

    // Configure the MPU for user challenges
    // The MPU makes unprivileged mode only have access to its RAM from 0x20008000 to 0x20020000.
    unsafe {
        // Region 0: RAM from 0x20000000 to 0x20020000 with first two subregions disabled
        p.core.MPU.rbar.write(0x20000000 | 0b10000 | 0);
        p.core.MPU.rasr.write(0b000_0_0_011_00_000110_00000011_00_10000_1);
        // Region 1: Give the user code access to USART6
        p.core.MPU.rbar.write(0x40011400 | 0b10000 | 1);
        p.core.MPU.rasr.write(0b000_1_0_011_00_000101_00000000_00_01001_1);
        // Region 2: Give the user code access to USART2 as well.
        // It is their fault if they break the configuration.
        p.core.MPU.rbar.write(0x40004400 | 0b10000 | 2);
        p.core.MPU.rasr.write(0b000_1_0_011_00_000101_00000000_00_01001_1);
        // Region 3: Allow the user code access to flash from 0x08000000 to 0x080C0000.
        // This allows accessing and dumping the main program but not the secret sectors
        // with keys and answers.
        p.core.MPU.rbar.write(0x08000000 | 0b10000 | 3);
        p.core.MPU.rasr.write(0b000_0_0_010_00_000010_11000000_00_10011_1);
        // Region 4-6: Disable
        p.core.MPU.rbar.write(0b10000 | 4);
        p.core.MPU.rasr.write(0);
        p.core.MPU.rbar.write(0b10000 | 5);
        p.core.MPU.rasr.write(0);
        p.core.MPU.rbar.write(0b10000 | 6);
        p.core.MPU.rasr.write(0);
        // Region 7: Stack overflow protection
        p.core.MPU.rbar.write(BKPSRAM_ADDR_0 | 0b10000 | 7);
        p.core.MPU.rasr.write(0b000_1_0_000_00_000110_00000000_00_00100_1);
        // Enable MPU and background region
        p.core.MPU.ctrl.write(0b101);
    }

    // Set up the RNG
    let rng = p.device.RNG;
    rng.cr.modify(|_, w| w.rngen().bit(true));
    while !rng.sr.read().drdy().bit() {}
    let rng_last_val = rng.dr.read().bits();

    // Large state variables
    let large_var_addr = 0x080C0000;
    let large_state_var = unsafe { &*(large_var_addr as *const PuzzlesStateLorge) };

    // Load settings into "secure" SRAM
    assert!(core::mem::size_of::<PuzzlesStateSmol>() < 0x100);
    let save_data_valid = unsafe {
        let save_data = core::slice::from_raw_parts(0x080E0000 as *const u8,
            core::mem::size_of::<PuzzlesStateSmol>());
        let save_computed_crc = crc::crc32::checksum_ieee(save_data);
        let save_stored_crc = *((0x080E0000 + core::mem::size_of::<PuzzlesStateSmol>()) as *const u32);

        save_computed_crc == save_stored_crc
    };
    let small_state_var = unsafe { &mut *(BKPSRAM_ADDR_3 as *mut PuzzlesStateSmol) };
    unsafe {
        core::ptr::copy_nonoverlapping(0x080E0000 as *const PuzzlesStateSmol, small_state_var, 1);
    }
    // Bandaid: bits for cleared levels are inverted to prevent reset glitch
    small_state_var.cleared_levels ^= 0xFFFFFFFF;

    // Command line ringbuffers
    let (cmdline_rx_p, cmdline_rx_c) = r.CMDLINE_RX_RB.split();
    let (mut cmdline_ret_p, cmdline_ret_c) = r.CMDLINE_RET_RB.split();

    cmdline_ret_p.enqueue(r.WORK_CMDLINE_BUF_0).unwrap();
    cmdline_ret_p.enqueue(r.WORK_CMDLINE_BUF_1).unwrap();

    init::LateResources {
        GPIOB: gpiob,
        TIM7_OBJ: tim7,
        USER_UART: usart2,
        CPLD_UART: usart6,

        IDLE_MS_COUNTER: r.MS_COUNTER,
        TIM7_MS_COUNTER: r.MS_COUNTER,

        CMDLINE_RX_P: cmdline_rx_p,
        CMDLINE_RX_C: cmdline_rx_c,
        CMDLINE_RET_P: cmdline_ret_p,
        CMDLINE_RET_C: cmdline_ret_c,

        RNG: rng,
        RNG_LAST_VAL: rng_last_val,

        DEBUG_MODE: debugmode,
        SAVE_DATA_VALID: save_data_valid,

        PUZZLES_STATE: small_state_var,
        PUZZLES_STATE_LORGE: large_state_var,

        SPEAKER_TIMER: tim2,

        LEVEL7_MODULUS: cryptastic::Bignum2048(large_state_var.level7_modulus),
        LEVEL7_R: cryptastic::Bignum2048(large_state_var.level7_r),
        LEVEL7_RR: cryptastic::Bignum2048(large_state_var.level7_rr),

        LEVEL8_PUBK: cryptastic::ECPointAffine256 {
            x: cryptastic::Bignum256(large_state_var.level8_pubk_x),
            y: cryptastic::Bignum256(large_state_var.level8_pubk_y),
        },
        LEVEL8_PUBK_PLUS_G: cryptastic::ECPointAffine256 {
            x: cryptastic::Bignum256(large_state_var.level8_pubk_plus_g_x),
            y: cryptastic::Bignum256(large_state_var.level8_pubk_plus_g_y),
        },
    }
}

pub fn hwrng_get_u32(rng: &stm32f405::RNG, last_val: &mut u32) -> u32 {
    loop {
        let mut sr = rng.sr.read();
        loop {
            // If a clock error happens, shrug but the data can be used
            if sr.ceis().bit() {
                rng.sr.modify(|_, w| w.ceis().bit(false));
            }

            // If there is a seed error, the data is not usable. Datasheet says to reset the RNG.
            if sr.seis().bit() {
                rng.sr.modify(|_, w| w.seis().bit(false));
                rng.cr.modify(|_, w| w.rngen().bit(false));
                rng.cr.modify(|_, w| w.rngen().bit(true));
                sr = rng.sr.read();
                continue;
            }

            if sr.drdy().bit() {
                break;
            }
            // Not ready yet
            sr = rng.sr.read();
        }
        // Data is ready
        let rngval = rng.dr.read().bits();

        // Ensure the data changed
        if rngval != *last_val {
            *last_val = rngval;
            return rngval;
        }
    }
}

pub fn hwrng_gen_pin(rng: &stm32f405::RNG, last_val: &mut u32) -> [u8; 4] {
    let mut ret = [0u8; 4];

    let pin = loop {
        let mut val32 = hwrng_get_u32(rng, last_val);
        if val32 < 4294960000 {
            break val32 % 10000;
        }
    };
    ret[3] = (pin % 10) as u8;
    ret[2] = ((pin % 100) / 10) as u8;
    ret[1] = ((pin % 1000) / 100) as u8;
    ret[0] = (pin / 1000) as u8;

    ret
}

pub fn msleep(ctr: &'static AtomicU32, ms: u32) {
    let time_start = ctr.load(Ordering::Relaxed);
    loop {
        let time_now = ctr.load(Ordering::Relaxed);
        if time_now.wrapping_sub(time_start) >= ms {
            break;
        }
        rtfm::wfi();
    }
}

fn level_to_earliest_valid(cleared_levels: u32, level_to_start: u32) -> u32 {
    if cleared_levels & (1 << 0) == 0 {
        0
    } else {
        level_to_start
    }
}

fn idle(t: &mut Threshold, mut r: idle::Resources) -> ! {
    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "> ");
        });

        let cmdline = uart_get_line_blocking(r.CMDLINE_RX_C, r.CMDLINE_RET_P, || () );
        let cmdline = cmdline.trim();
        let mut cmdline_split = cmdline.split_whitespace();
        let cmd0 = cmdline_split.next();

        if cmd0.is_some() {
            let cmd0 = cmd0.unwrap();
            match cmd0 {
                "__lockme" => {
                    flash_option_lock();
                },
                "__unlockme_this_will_brick_the_device" => {
                    flash_option_unlock();
                },
                "__reset_cleared_levels" => {
                    if !*r.SAVE_DATA_VALID {
                        // If the save data is corrupted, we need to generate new PINs to fix it
                        r.PUZZLES_STATE.level0_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                        r.PUZZLES_STATE.level1_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                        r.PUZZLES_STATE.level2_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                        r.PUZZLES_STATE.level2_tries_left = 3;

                        r.PUZZLES_STATE.enable_sound = true;

                        *r.SAVE_DATA_VALID = true;
                    }

                    r.PUZZLES_STATE.last_cleared_level = -1;
                    r.PUZZLES_STATE.cleared_levels = 0;
                    flash_save_settings(r.PUZZLES_STATE, true);
                },
                "sound" => {
                    if !*r.SAVE_DATA_VALID {
                        r.USER_UART.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "ERROR: Save data is corrupted!\r\n");
                        });
                        continue;
                    }

                    if let Some(cmd1) = cmdline_split.next() {
                        match cmd1 {
                            "on" | "yes" | "true" => {
                                r.PUZZLES_STATE.enable_sound = true;
                            },
                            "off" | "no" | "false" => {
                                r.PUZZLES_STATE.enable_sound = false;
                            },
                            _ => {},
                        }
                    }

                    flash_save_settings(r.PUZZLES_STATE, true);

                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Sound effects are ");
                        if r.PUZZLES_STATE.enable_sound {
                            uart_putstr(user_uart, "on\r\n");
                        } else {
                            uart_putstr(user_uart, "off\r\n");
                        }
                    });
                }
                "cpld_get_idcode" => {
                    let idcode = cpld_get_idcode(r.GPIOB);

                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putu32(user_uart, idcode);
                        uart_putstr(user_uart, "\r\n");
                    });
                },
                "cpld_read_eeprom" => {
                    cpld_read_eeprom(r.GPIOB, r.IDLE_MS_COUNTER, |_, bitidx, bit| {
                        r.USER_UART.claim(t, |user_uart, _t| {
                            if bit {
                                uart_putc!(user_uart, '1');
                            } else {
                                uart_putc!(user_uart, '0');
                            }

                            if bitidx == 259 {
                                uart_putstr(user_uart, "\r\n");
                            }
                        });
                    });
                },
                "cpld_erase_eeprom" => {
                    cpld_erase(r.GPIOB, r.IDLE_MS_COUNTER);
                },
                "cpld_write_eeprom" => {
                    r.USER_UART.claim(t, |user_uart, _t| {
                        // Disable the UART RX interrupt. This disables the line editing
                        // and allows us to directly read characters
                        user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

                        cpld_write_eeprom(r.GPIOB, r.IDLE_MS_COUNTER, |_, _| {
                            let c = uart_getc_manual(user_uart);
                            c == '1' as u8
                        });

                        // Need to reenable interrupt
                        user_uart.cr1.modify(|_, w| w.rxneie().bit(true));
                    });
                },
                "help" => {
                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Commands:\r\n\
                            help -- Display this help\r\n\
                            version -- Display the firmware version\r\n\
                            start <level> -- start the next level or the chosen level\r\n\
                            next -- start the next level that isn't cleared\r\n\
                            status -- display status of levels\r\n\
                            flags -- display unlocked flags\r\n\
                            sound on|off -- enable/disable sound effects\r\n\
                            serialno -- prints the serial number of the board\r\n\
                            cpld_get_idcode -- Read the IDCODE of the CPLD\r\n\
                            cpld_read_eeprom -- Read the internal EEPROM of the CPLD\r\n\
                            cpld_erase_eeprom -- Erase the EEPROM of the CPLD\r\n\
                            cpld_write_eeprom -- Program data into the EEPROM of the CPLD\r\n");

                        uart_putstr(user_uart, "\r\n");
                    });
                },
                "version" => {
                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, env!("GIT_HASH"));
                        uart_putstr(user_uart, "\r\n");
                    });
                },
                "status" => {
                    if !*r.SAVE_DATA_VALID {
                        r.USER_UART.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "ERROR: Save data is corrupted!\r\n");
                        });
                        continue;
                    }

                    r.USER_UART.claim(t, |user_uart, _t| {
                        if *r.DEBUG_MODE {
                            uart_putstr(user_uart, "\x1b[31mYOU ARE USING AN UNLOCKED DEBUG BOARD!\x1b[0m\r\n");
                        }

                        uart_putstr(user_uart, "Currently cleared levels:\r\n");
                        for i in 0..16u32 {
                            uart_putstr(user_uart, "  Level ");
                            uart_putstr(user_uart, &i.to_string());

                            // Tagline
                            uart_putstr(user_uart, match i {
                                0 => " - \"It's just like in the movies!\" (application vuln)",
                                1 => " - \"Let's get serious\" (application vuln)",
                                2 => " - \"Stop bruteforcing me!\" (application vuln)",
                                3 => " - \"Let's play a game\" (application vuln)",
                                4 => " - \"Play by the rules\" (crypto vuln)",
                                5 => " - \"Cosmic rays\" (crypto vuln)",
                                6 => " - \"Consultants fixed our RNG\" (exploitation)",
                                7 => " - \"No twiizers required\" (crypto vuln)",
                                8 => " - \"not success overflow\" (crypto vuln)",
                                9 => " - \"secure firmware loading\" (crypto vuln)",
                                10 => " - \"m4loaderhax\" (crypto vuln)",
                                11 => " - \"Secure Enclave\" (application vuln)",
                                12 => " - \"Secure Enclave, redux\" (application vuln)",
                                13 => " - \"CPLD reverse engineering?\" (reversing)",
                                14 => " - \"CPLD reverse engineering! Part 2\" (reversing)",
                                15 => " - \"CPLD reverse engineering - the final puzzle\" (reversing)",
                                _ => unreachable!(),
                            });

                            if r.PUZZLES_STATE.cleared_levels & (1 << i) != 0 {
                                uart_putstr(user_uart, ": ✅\r\n");
                            } else {
                                uart_putstr(user_uart, ": ❌\r\n");
                            }
                        }
                    });
                }
                "flags" => {
                    if !*r.SAVE_DATA_VALID {
                        r.USER_UART.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "ERROR: Save data is corrupted!\r\n");
                        });
                        continue;
                    }

                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Currently cleared levels:\r\n");
                        for i in 0..16u32 {
                            uart_putstr(user_uart, "  Level ");
                            uart_putstr(user_uart, &i.to_string());

                            if r.PUZZLES_STATE.cleared_levels & (1 << i) != 0 {
                                uart_putstr(user_uart, ": ");
                                for j in 0..32 {
                                    uart_putc!(user_uart, r.PUZZLES_STATE_LORGE.flags[i as usize * 32 + j]);
                                }
                                uart_putstr(user_uart, "\r\n");
                            } else {
                                uart_putstr(user_uart, ": ❌\r\n");
                            }
                        }
                    });
                }
                "start" | "next" => {
                    if !*r.SAVE_DATA_VALID {
                        r.USER_UART.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "ERROR: Save data is corrupted!\r\n");
                        });
                        continue;
                    }

                    // Default level is one after the last cleared one
                    let mut level_to_start = r.PUZZLES_STATE.last_cleared_level + 1;

                    if r.PUZZLES_STATE.cleared_levels == 0xFFFF {
                        // If everything is cleared then set to special -1
                        level_to_start = -1;
                    } else {
                        // If this level is already cleared, find the lowest uncleared level.
                        if level_to_start > 15 || r.PUZZLES_STATE.cleared_levels & (1 << level_to_start) != 0 {
                            for i in 0..15 {
                                if r.PUZZLES_STATE.cleared_levels & (1 << i) == 0 {
                                    level_to_start = i;
                                    break;
                                }
                            }
                        }
                    }

                    // Now if the user entered something, do that instead
                    if let Some(cmd1) = cmdline_split.next() {
                        if let Ok(start_level_parsed) = cmd1.parse::<u32>() {
                            if start_level_parsed <= 15 {
                                level_to_start = start_level_parsed as i32;
                            } else {
                                r.USER_UART.claim(t, |user_uart, _t| {
                                    uart_putstr(user_uart, "Invalid level, starting next level...\r\n");
                                });
                            }
                        } else {
                            r.USER_UART.claim(t, |user_uart, _t| {
                                uart_putstr(user_uart, "Invalid level, starting next level...\r\n");
                            });
                        }
                    }

                    // If the level to start is still -1, then there are no levels left but the
                    // user did not force one either.
                    if level_to_start == -1 {
                        r.USER_UART.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "No more levels! You beat the game!\r\n");
                        });
                    } else {
                        // Now filter for things that the user isn't allowed to start yet
                        let mut level_to_start = level_to_start as u32;
                        let new_level_to_start = level_to_earliest_valid(r.PUZZLES_STATE.cleared_levels, level_to_start);
                        if new_level_to_start != level_to_start {
                            r.USER_UART.claim(t, |user_uart, _t| {
                                uart_putstr(user_uart, "Level not unlocked! Starting prerequisite level...\r\n");
                            });
                        }
                        level_to_start = new_level_to_start;

                        let (should_save, was_successful) = match level_to_start {
                            0 => { level0(t, &mut r) },
                            1 => { level1(t, &mut r) },
                            2 => { level2(t, &mut r) },
                            3 => { level3(t, &mut r) },
                            4 => { level4(t, &mut r) },
                            5 => { level5(t, &mut r) },
                            6 => { level6(t, &mut r) },
                            7 => { level7(t, &mut r) },
                            8 => { level8(t, &mut r) },
                            9 => { level9(t, &mut r) },
                            10 => { level10(t, &mut r) },
                            11 => { level11(t, &mut r) },
                            12 => { level12(t, &mut r) },
                            13 => { level13(t, &mut r) },
                            14 => { level14(t, &mut r) },
                            15 => { level15(t, &mut r) },
                            _ => unreachable!(),
                        };

                        if should_save {
                            flash_save_settings(r.PUZZLES_STATE, true);
                        }

                        if was_successful && r.PUZZLES_STATE.enable_sound {
                            play_song(&r, FOUR_NOTE_COMPLETE_SFX);
                        }
                    }
                },
                "serialno" => {
                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "DEVICE_ID: ");
                        let devid = get_device_id_hex();
                        for &c in devid.iter() {
                            uart_putc!(user_uart, c);
                        }
                        uart_putstr(user_uart, "\r\n");
                    });
                }
                "__provision" => {
                    {
                        let small_puzzle_state = &mut r.PUZZLES_STATE;
                        let large_state_var = unsafe { &mut *(0x20008000 as *mut PuzzlesStateLorge) };

                        // Misc.
                        small_puzzle_state.last_cleared_level = -1;
                        small_puzzle_state.cleared_levels = 0;
                        small_puzzle_state.enable_sound = true;

                        // PIN codes
                        small_puzzle_state.level0_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                        small_puzzle_state.level1_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                        small_puzzle_state.level2_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                        small_puzzle_state.level2_tries_left = 3;

                        r.USER_UART.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "DEVICE_ID: ");
                            let devid = get_device_id_hex();
                            for &c in devid.iter() {
                                uart_putc!(user_uart, c);
                            }
                            uart_putstr(user_uart, "\r\n");

                            // Disable the UART RX interrupt. This disables the line editing
                            // and allows us to directly read characters
                            user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

                            // RNG state
                            for i in 0..large_state_var.level4_seed.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level4_seed[large_state_var.level4_seed.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');

                            // RSA modulus
                            for i in 0..large_state_var.level7_modulus.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level7_modulus[large_state_var.level7_modulus.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');

                            // RSA R
                            for i in 0..large_state_var.level7_r.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level7_r[large_state_var.level7_r.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');

                            // RSA R^2
                            for i in 0..large_state_var.level7_rr.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level7_rr[large_state_var.level7_rr.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');

                            // RSA example
                            for i in 0..large_state_var.level7_example.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level7_example[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // ECDSA pubk
                            for i in 0..large_state_var.level8_pubk_x.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level8_pubk_x[large_state_var.level8_pubk_x.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');
                            for i in 0..large_state_var.level8_pubk_y.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level8_pubk_y[large_state_var.level8_pubk_y.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');

                            // ECDSA pubk + G
                            for i in 0..large_state_var.level8_pubk_plus_g_x.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level8_pubk_plus_g_x[large_state_var.level8_pubk_plus_g_x.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');
                            for i in 0..large_state_var.level8_pubk_plus_g_y.len() {
                                let c1 = uart_getc_manual(user_uart);
                                let c2 = uart_getc_manual(user_uart);
                                let c3 = uart_getc_manual(user_uart);
                                let c4 = uart_getc_manual(user_uart);
                                large_state_var.level8_pubk_plus_g_y[large_state_var.level8_pubk_plus_g_y.len() - 1 - i] =
                                    ((c1 as u32) << 24) |
                                    ((c2 as u32) << 16) |
                                    ((c3 as u32) << 8) |
                                    (c4 as u32);
                            }
                            uart_putc!(user_uart, 'O');

                            // ECDSA example 1
                            for i in 0..large_state_var.level8_example1.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level8_example1[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // ECDSA example 2
                            for i in 0..large_state_var.level8_example2.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level8_example2[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // Level 9 code
                            for i in 0..large_state_var.level9_code.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level9_code[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // Level 9 XOR
                            for i in 0..large_state_var.level9_xor.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level9_xor[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // Level 10 code
                            for i in 0..large_state_var.level10_code.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level10_code[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // Level 10 key
                            for i in 0..large_state_var.level10_key_orig.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.level10_key_orig[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // CPLD 16-bit answer
                            for i in 0..large_state_var.cpld_16bit_answer.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.cpld_16bit_answer[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // CPLD 64-bit answer
                            for i in 0..large_state_var.cpld_64bit_answer.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.cpld_64bit_answer[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // CPLD "center" answer
                            for i in 0..large_state_var.cpld_center_hidden.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.cpld_center_hidden[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // CPLD "pterm" answer
                            for i in 0..large_state_var.cpld_pterm_hidden.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.cpld_pterm_hidden[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // CTF flags
                            for i in 0..large_state_var.flags.len() {
                                let c = uart_getc_manual(user_uart);
                                large_state_var.flags[i] = c;
                            }
                            uart_putc!(user_uart, 'O');

                            // Need to reenable interrupt
                            user_uart.cr1.modify(|_, w| w.rxneie().bit(true));
                        });
                    }

                    flash_save_settings(r.PUZZLES_STATE, false);
                },
                _ => {
                    r.USER_UART.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Unrecognized command!\r\n");
                    });
                },
            }
        }
    }
}

fn millisecond_intr(_t: &mut Threshold, r: TIM7::Resources) {
    r.TIM7_OBJ.sr.modify(|_, w| w.uif().clear());
    r.TIM7_MS_COUNTER.fetch_add(1, Ordering::Relaxed);
}

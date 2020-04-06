// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use core;
use core::fmt::Write;
use core::intrinsics;
use cortex_m_semihosting::hio;
use stm32f405;

#[macro_use]
use cli::*;

use ::{CALL_PAYLOAD_STASHED_CTX, LEVEL9_CLEAR_FLAG, LEVEL10_CLEAR_FLAG};

#[repr(C)]
pub struct SVCallCTX {
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r0dummy: u32,
    pub exc_return: u32,
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r12: u32,
    pub lr: u32,
    pub pc: u32,
    pub xpsr: u32,
}

#[lang = "panic_fmt"]
#[no_mangle]
pub unsafe extern "C" fn rust_begin_unwind(
    args: core::fmt::Arguments,
    file: &'static str,
    line: u32,
    col: u32,
) -> ! {
    if let Ok(mut stdout) = hio::hstdout() {
        write!(stdout, "panicked at '")
            .and_then(|_| {
                stdout
                    .write_fmt(args)
                    .and_then(|_| writeln!(stdout, "', {}:{}:{}", file, line, col))
            })
            .ok();
    }

    intrinsics::abort()
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn SVCALL() {
    asm!("
        push {r0, lr}
        push {r4-r11}
        mov r0, sp
        bl $0
        mov sp, r0
        pop {r4-r11}
        pop {r0, pc}
        "
        :
        : "i"(svcall_handler as unsafe extern "C" fn(*mut SVCallCTX) -> *mut SVCallCTX)
        :
        : "volatile");
}

unsafe fn svcall_dump_ctx(ctx: *mut SVCallCTX) {
    let uart = &*stm32f405::USART2::ptr();

    uart_putstr(uart, "\r\n\r\n********************************************************************************\r\n");
    uart_putstr(uart,         "*                           AN ILLEGAL SVC OCCURRED                            *\r\n");
    uart_putstr(uart,         "********************************************************************************\r\n");

    uart_putstr(uart, "R0 = ");
    uart_putu32(uart, (*ctx).r0);
    uart_putstr(uart, "\r\nR1 = ");
    uart_putu32(uart, (*ctx).r1);
    uart_putstr(uart, "\r\nR2 = ");
    uart_putu32(uart, (*ctx).r2);
    uart_putstr(uart, "\r\nR3 = ");
    uart_putu32(uart, (*ctx).r3);
    uart_putstr(uart, "\r\nR4 = ");
    uart_putu32(uart, (*ctx).r4);
    uart_putstr(uart, "\r\nR5 = ");
    uart_putu32(uart, (*ctx).r5);
    uart_putstr(uart, "\r\nR6 = ");
    uart_putu32(uart, (*ctx).r6);
    uart_putstr(uart, "\r\nR7 = ");
    uart_putu32(uart, (*ctx).r7);
    uart_putstr(uart, "\r\nR8 = ");
    uart_putu32(uart, (*ctx).r8);
    uart_putstr(uart, "\r\nR9 = ");
    uart_putu32(uart, (*ctx).r9);
    uart_putstr(uart, "\r\nR10 = ");
    uart_putu32(uart, (*ctx).r10);
    uart_putstr(uart, "\r\nR11 = ");
    uart_putu32(uart, (*ctx).r11);
    uart_putstr(uart, "\r\nR12 = ");
    uart_putu32(uart, (*ctx).r12);
    uart_putstr(uart, "\r\nSP (with exception frame) = ");
    uart_putu32(uart, ctx as u32);
    uart_putstr(uart, "\r\nLR = ");
    uart_putu32(uart, (*ctx).lr);
    uart_putstr(uart, "\r\nPC = ");
    uart_putu32(uart, (*ctx).pc);
    uart_putstr(uart, "\r\nxPSR = ");
    uart_putu32(uart, (*ctx).xpsr);
}

pub extern "C" fn invoke_user_code(code_addr: u32, return_addr: u32) -> u32 {
    let ret;
    unsafe {
        asm!("
            mov r2, $2
            mov r1, $1
            movs r0, #0
            movt r0, #0xAA55
            svc 0
            mov $0, r0"
            : "=r"(ret)
            : "r"(code_addr), "r"(return_addr)
            : "r0", "r1", "r2"
            : "volatile");
    }
    ret
}

unsafe extern "C" fn svcall_handler(ctx: *mut SVCallCTX) -> *mut SVCallCTX {
    // Stack contents are:
    //  R4-R11
    //  R0 (dummy)
    //  EXC_RETURN
    //  R0-R3
    //  R12
    //  LR
    //  PC
    //  xPSR
    let ctx = &mut *ctx;
    let req_func = ctx.r0;
    let uart = &*stm32f405::USART2::ptr();

    match req_func {
        0xAA550000 => {
            // Jump to unprivileged code

            // Cannot do this jump if currently unprivileged
            if !CALL_PAYLOAD_STASHED_CTX.is_null() {
                svcall_dump_ctx(ctx);
                loop {}
            }

            CALL_PAYLOAD_STASHED_CTX = ctx;
            let user_func_addr = ctx.r1;
            let user_func_lr = ctx.r2;

            let ctx = &mut *((0x20020000 - core::mem::size_of::<SVCallCTX>()) as *mut SVCallCTX);

            ctx.pc = user_func_addr;
            ctx.lr = user_func_lr;

            // Clear everything else
            ctx.r4 = 0;
            ctx.r5 = 0;
            ctx.r6 = 0;
            ctx.r7 = 0;
            ctx.r8 = 0;
            ctx.r9 = 0;
            ctx.r10 = 0;
            ctx.r11 = 0;
            ctx.r0dummy = 0;
            ctx.exc_return = 0xFFFFFFF9;
            ctx.r0 = 0;
            ctx.r1 = 0;
            ctx.r2 = 0;
            ctx.r3 = 0;
            ctx.r12 = 0;
            ctx.xpsr = 0x01000000;

            // After return, will be in unprivileged mode
            asm!("
                mrs r0, CONTROL
                orr r0, #1
                msr CONTROL, r0"
                :
                :
                : "r0"
                : "volatile");

            ctx
        },
        0xDEADBEEF => {
            // Arbitrary code exec simulation win syscall

            if ctx.r1 != 0x556e6c6f {       // 'Unlo'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r2 != 0x636b204c {       // 'ck L'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r3 != 0x6576656c {       // 'evel'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r4 != 0x20392070 {       // ' 9 p'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r5 != 0x6c30783f {       // 'l0x?'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r6 != 0x206b7468 {       // ' kth'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r7 != 0x78626169 {       // 'xbai'
                svcall_dump_ctx(ctx);
                loop {}
            }

            // Ok!
            LEVEL9_CLEAR_FLAG = true;
            ctx
        },
        0xDEADFACE => {
            // Arbitrary code exec simulation win syscall

            if ctx.r1 != 0x49206361 {       // 'I ca'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r2 != 0x6e206861 {       // 'n ha'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r3 != 0x7a206c65 {       // 'z le'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r4 != 0x76656c20 {       // 'vel '
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r5 != 0x31302063 {       // '10 c'
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r6 != 0x6f646520 {       // 'ode '
                svcall_dump_ctx(ctx);
                loop {}
            }
            if ctx.r7 != 0x65786563 {       // 'exec'
                svcall_dump_ctx(ctx);
                loop {}
            }

            // Ok!
            LEVEL10_CLEAR_FLAG = true;
            ctx
        },
        0 => {
            let retval = ctx.r1;
            let ctx = CALL_PAYLOAD_STASHED_CTX;
            (*ctx).r0 = retval;
            CALL_PAYLOAD_STASHED_CTX = core::ptr::null_mut();

            // After return, will be in privileged mode
            asm!("
                mrs r0, CONTROL
                bic r0, #1
                msr CONTROL, r0"
                :
                :
                : "r0"
                : "volatile");

            ctx
        },
        1 => {
            let to_output_val = ctx.r1;
            uart_putu32(uart, to_output_val);
            ctx
        },
        2 => {
            let to_output_val = ctx.r1;
            uart_putu8(uart, to_output_val as u8);
            ctx
        },
        3 => {
            let to_output_val = ctx.r1;
            uart_putc!(uart, to_output_val as u8);
            ctx
        },
        _ => {
            svcall_dump_ctx(ctx);
            loop {}
        }
    }
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn DEFAULT_HANDLER() -> ! {
    asm!("
        push {r4-r11}
        mrs r0, psp
        push {r0}
        // Useless, needed only to maintain 8 byte alignment
        push {r0}

        mov r1, lr
        tst lr, #0x10
        bne 1f

        vpush.32 {s16-s31}

        1:
        mov r0, sp
        b $0
        "
        :
        : "i"(debug_handler as unsafe extern "C" fn(*mut u32, u32) -> !)
        :
        : "volatile");

    loop {}
}

unsafe extern "C" fn debug_handler(ef: *mut u32, lr: u32) -> ! {
    let uart = &*stm32f405::USART2::ptr();

    uart_putstr(uart, "\r\n\r\n********************************************************************************\r\n");
    uart_putstr(uart,         "*                  AN UNEXPECTED EXCEPTION/INTERRUPT OCCURRED                  *\r\n");
    uart_putstr(uart,         "********************************************************************************\r\n");

    if lr & 0x10 != 0 {
        // No FPU state
        // Stack contents are:
        //  PSP, PSP
        //  R4-R11
        //  R0-R3
        //  R12
        //  LR
        //  PC
        //  xPSR
        uart_putstr(uart, "R0 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(10)));
        uart_putstr(uart, "\r\nR1 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(11)));
        uart_putstr(uart, "\r\nR2 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(12)));
        uart_putstr(uart, "\r\nR3 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(13)));
        uart_putstr(uart, "\r\nR4 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(2)));
        uart_putstr(uart, "\r\nR5 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(3)));
        uart_putstr(uart, "\r\nR6 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(4)));
        uart_putstr(uart, "\r\nR7 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(5)));
        uart_putstr(uart, "\r\nR8 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(6)));
        uart_putstr(uart, "\r\nR9 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(7)));
        uart_putstr(uart, "\r\nR10 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(8)));
        uart_putstr(uart, "\r\nR11 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(9)));
        uart_putstr(uart, "\r\nR12 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(14)));
        uart_putstr(uart, "\r\nMSP (with exception frame) = ");
        uart_putu32(uart, ef as u32);
        uart_putstr(uart, "\r\nPSP = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(0)));
        uart_putstr(uart, "\r\nLR = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(15)));
        uart_putstr(uart, "\r\nPC = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(16)));
        uart_putstr(uart, "\r\nxPSR = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(17)));
    } else {
        // With FPU state
        // Stack contents are:
        //  S16-S31
        //  PSP, PSP
        //  R4-R11
        //  R0-R3
        //  R12
        //  LR
        //  PC
        //  xPSR
        //  S0-S15
        //  FPSCR
        uart_putstr(uart, "R0 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(26)));
        uart_putstr(uart, "\r\nR1 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(27)));
        uart_putstr(uart, "\r\nR2 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(28)));
        uart_putstr(uart, "\r\nR3 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(29)));
        uart_putstr(uart, "\r\nR4 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(18)));
        uart_putstr(uart, "\r\nR5 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(19)));
        uart_putstr(uart, "\r\nR6 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(20)));
        uart_putstr(uart, "\r\nR7 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(21)));
        uart_putstr(uart, "\r\nR8 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(22)));
        uart_putstr(uart, "\r\nR9 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(23)));
        uart_putstr(uart, "\r\nR10 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(24)));
        uart_putstr(uart, "\r\nR11 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(25)));
        uart_putstr(uart, "\r\nR12 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(30)));
        uart_putstr(uart, "\r\nMSP (with exception frame) = ");
        uart_putu32(uart, ef as u32);
        uart_putstr(uart, "\r\nPSP = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(16)));
        uart_putstr(uart, "\r\nLR = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(31)));
        uart_putstr(uart, "\r\nPC = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(32)));
        uart_putstr(uart, "\r\nxPSR = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(33)));

        uart_putstr(uart, "\r\n\r\nS0 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(34)));
        uart_putstr(uart, "\r\nS1 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(35)));
        uart_putstr(uart, "\r\nS2 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(36)));
        uart_putstr(uart, "\r\nS3 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(37)));
        uart_putstr(uart, "\r\nS4 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(38)));
        uart_putstr(uart, "\r\nS5 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(39)));
        uart_putstr(uart, "\r\nS6 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(40)));
        uart_putstr(uart, "\r\nS7 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(41)));
        uart_putstr(uart, "\r\nS8 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(42)));
        uart_putstr(uart, "\r\nS9 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(43)));
        uart_putstr(uart, "\r\nS10 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(44)));
        uart_putstr(uart, "\r\nS11 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(45)));
        uart_putstr(uart, "\r\nS12 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(46)));
        uart_putstr(uart, "\r\nS13 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(47)));
        uart_putstr(uart, "\r\nS14 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(48)));
        uart_putstr(uart, "\r\nS15 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(49)));
        uart_putstr(uart, "\r\nS16 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(0)));
        uart_putstr(uart, "\r\nS17 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(1)));
        uart_putstr(uart, "\r\nS18 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(2)));
        uart_putstr(uart, "\r\nS19 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(3)));
        uart_putstr(uart, "\r\nS20 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(4)));
        uart_putstr(uart, "\r\nS21 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(5)));
        uart_putstr(uart, "\r\nS22 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(6)));
        uart_putstr(uart, "\r\nS23 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(7)));
        uart_putstr(uart, "\r\nS24 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(8)));
        uart_putstr(uart, "\r\nS25 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(9)));
        uart_putstr(uart, "\r\nS26 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(10)));
        uart_putstr(uart, "\r\nS27 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(11)));
        uart_putstr(uart, "\r\nS28 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(12)));
        uart_putstr(uart, "\r\nS29 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(13)));
        uart_putstr(uart, "\r\nS30 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(14)));
        uart_putstr(uart, "\r\nS31 = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(15)));
        uart_putstr(uart, "\r\nFPCSR = ");
        uart_putu32(uart, core::ptr::read_volatile(ef.offset(50)));
    }

    uart_putstr(uart, "\r\n\r\nCFSR = ");
    uart_putu32(uart, core::ptr::read_volatile(0xE000ED28 as *mut u32));
    uart_putstr(uart, "\r\nHFSR = ");
    uart_putu32(uart, core::ptr::read_volatile(0xE000ED2C as *mut u32));
    uart_putstr(uart, "\r\nMMFAR = ");
    uart_putu32(uart, core::ptr::read_volatile(0xE000ED34 as *mut u32));
    uart_putstr(uart, "\r\nBFAR = ");
    uart_putu32(uart, core::ptr::read_volatile(0xE000ED38 as *mut u32));
    uart_putstr(uart, "\r\nAFSR = ");
    uart_putu32(uart, core::ptr::read_volatile(0xE000ED3C as *mut u32));
    uart_putstr(uart, "\r\n");

    loop {}
}

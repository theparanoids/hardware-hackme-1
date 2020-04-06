// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use rtfm::{Threshold};
use alloc::String;
use alloc::borrow::ToOwned;
use heapless::ring_buffer::{Consumer, Producer};
use core::str;

pub const HEXLUT: [u8; 16] = [
    '0' as u8,
    '1' as u8,
    '2' as u8,
    '3' as u8,
    '4' as u8,
    '5' as u8,
    '6' as u8,
    '7' as u8,
    '8' as u8,
    '9' as u8,
    'A' as u8,
    'B' as u8,
    'C' as u8,
    'D' as u8,
    'E' as u8,
    'F' as u8];

macro_rules! uart_putc {
    ($uart:expr, $c:expr) => {{
        while !$uart.sr.read().txe().bit() {}
        $uart.dr.write(|w| w.dr().bits($c as u16));
    }}
}

pub fn uart_putstr(uart: &::stm32f405::usart6::RegisterBlock, s: &str) {
    for c in s.bytes() {
        uart_putc!(uart, c);
    }
}

pub fn uart_putu8(uart: &::stm32f405::usart6::RegisterBlock, x: u8) {
    let n = (x >> 4) & 0xF;
    uart_putc!(uart, HEXLUT[n as usize]);
    let n = x & 0xF;
    uart_putc!(uart, HEXLUT[n as usize]);
}

pub fn uart_putu32(uart: &::stm32f405::usart6::RegisterBlock, x: u32) {
    let b = x >> 24;
    uart_putu8(uart, b as u8);
    let b = x >> 16;
    uart_putu8(uart, b as u8);
    let b = x >> 8;
    uart_putu8(uart, b as u8);
    let b = x;
    uart_putu8(uart, b as u8);
}

pub fn uart_getc_manual(uart: &::stm32f405::usart6::RegisterBlock) -> u8 {
    let mut sr = uart.sr.read();
    while !sr.rxne().bit() || sr.nf().bit() || sr.fe().bit() {
        sr = uart.sr.read();
        if sr.nf().bit() || sr.fe().bit() {
            let _ = uart.dr.read();
        }
    }
    uart.dr.read().dr().bits() as u8
}

pub fn uart_getc_nowait(uart: &::stm32f405::usart6::RegisterBlock) -> Option<u8> {
    let sr = uart.sr.read();
    if !sr.rxne().bit() || sr.nf().bit() || sr.fe().bit() {
        if sr.nf().bit() || sr.fe().bit() {
            let _ = uart.dr.read();
        }

        return None;
    }
    Some(uart.dr.read().dr().bits() as u8)
}

pub struct CmdlineBuf {
    pub buf: [u8; 256],
    pub len: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LineEditEscapeFSM {
    NONE,
    CR,
    ESC,
    ESCLBRACKET,
}

pub struct LineEditState {
    pub buf: [u8; 256],
    pub pos: usize,
    pub len: usize,
    pub escape_fsm: LineEditEscapeFSM,
}

impl LineEditState {
    pub const fn new() -> Self {
        Self {
            buf: [0u8; 256],
            pos: 0,
            len: 0,
            escape_fsm: LineEditEscapeFSM::NONE,
        }
    }
}

pub fn uart_get_line_blocking<F>(
    rx_c: &mut Consumer<'static, &'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]>,
    tx_p: &mut Producer<'static, &'static mut CmdlineBuf, [&'static mut CmdlineBuf; 3]>,
    mut non_utf8_cb: F) -> String where F: FnMut() {

    loop {
        if let Some(cmdline_buf) = rx_c.dequeue() {
            let ret = if let Ok(cmdline) = str::from_utf8(&cmdline_buf.buf[0..cmdline_buf.len]) {
                let ret = cmdline.to_owned();
                Some(ret)
            } else {
                None
            };

            if let Some(ret) = ret {
                tx_p.enqueue(cmdline_buf).unwrap();
                return ret;
            } else {
                non_utf8_cb();
            }
        } else {
            ::rtfm::wfi();
        }
    }
}

pub fn uart_rx_lineedit(_t: &mut Threshold, mut r: ::USART2::Resources) {
    let sr = r.USER_UART.sr.read();
    let c = r.USER_UART.dr.read().dr().bits() as u8;

    // Ignore noise or framing errors. Assume overrun does not occur. Cannot get parity errors as
    // they are not enabled. Note that we read DR anyways to clear error flags.
    if sr.nf().bit() || sr.fe().bit() {
        return;
    }

    let mut state = r.LINEEDIT_STATE;

    // No matter if we get a \n, \r, or \r\n, always reply with \r\n

    if state.escape_fsm == LineEditEscapeFSM::CR {
        state.escape_fsm = LineEditEscapeFSM::NONE;

        // If we got \r\n skip this \n (otherwise proceed as normal)
        if c == '\n' as u8 {
            // Do not echo the newline! We injected a \n upon getting a \r
            return;
        }
    }

    match state.escape_fsm {
        LineEditEscapeFSM::NONE => {
            // Not processing an escape sequence

            // If newline we are done
            if c == '\r' as u8 || c == '\n' as u8 {
                if c == '\n' as u8 {
                    // Inject a \r
                    uart_putc!(r.USER_UART, '\r');
                }

                // Echo the newline
                uart_putc!(r.USER_UART, c);

                if c == '\r' as u8 {
                    state.escape_fsm = LineEditEscapeFSM::CR;
                    // Inject a \n
                    uart_putc!(r.USER_UART, '\n');
                }

                // Try to get a free buffer
                if let Some(free_buf) = r.CMDLINE_RET_C.dequeue() {
                    *free_buf = CmdlineBuf {
                        buf: state.buf,
                        len: state.len
                    };

                    // Because of the way we set up the queues, this is always going to succeed.
                    r.CMDLINE_RX_P.enqueue(free_buf).unwrap();
                }

                state.pos = 0;
                state.len = 0;
                return;
            }

            let pos = state.pos;
            let len = state.len;

            // A backspace (or a delete)!
            if c == 0x08u8 || c == 0x7f {
                if state.len == 0 || state.pos == 0 {
                    return;
                }

                // Echo a backspace explicitly (screen doesn't like 0x7f)
                uart_putc!(r.USER_UART, 0x08);

                if state.len != state.pos {
                    // Deleting a character from the middle, so we need to move everything left
                    for i in pos..len {
                        state.buf[i - 1] = state.buf[i];
                    }

                    // Need to reprint everything too
                    for i in pos-1..len-1 {
                        uart_putc!(r.USER_UART, state.buf[i]);
                    }
                }

                // Need this to actually wipe the (deleted/last) character away
                uart_putc!(r.USER_UART, ' ');
                uart_putc!(r.USER_UART, 0x08);

                // Need to adjust the cursor back
                for _ in pos-1..len-1 {
                    uart_putc!(r.USER_UART, 0x08);
                }

                state.pos -= 1;
                state.len -= 1;
                return;
            }

            // An escape!
            if c == 0x1bu8 {
                state.escape_fsm = LineEditEscapeFSM::ESC;
                return;
            }

            // Just inserting a normal character

            // If length is full, do nothing
            if state.len == state.buf.len() {
                return;
            }

            if state.len != state.pos {
                // Inserting something in the middle, so we need to move everything right
                for i in (pos..len).rev() {
                    state.buf[i + 1] = state.buf[i];
                }
            }

            // Echo the character
            uart_putc!(r.USER_UART, c);

            // If a character was inserted, we have to print everything else too
            if state.len != state.pos {
                for i in pos+1..len+1 {
                    uart_putc!(r.USER_UART, state.buf[i]);
                }
                // Now backspace to the same position
                for _ in pos+1..len+1 {
                    uart_putc!(r.USER_UART, 0x08);
                }
            }

            state.buf[pos] = c;
            state.pos += 1;
            state.len += 1;
        },
        LineEditEscapeFSM::CR => unreachable!(),
        LineEditEscapeFSM::ESC => {
            // Got ESC-[
            if c == '[' as u8 {
                state.escape_fsm = LineEditEscapeFSM::ESCLBRACKET;
            } else {
                // Got ESC-x which we don't understand. Just ignore it
                state.escape_fsm = LineEditEscapeFSM::NONE;
            }
        },
        LineEditEscapeFSM::ESCLBRACKET => {
            // Left arrow
            if c == 'D' as u8 {
                if state.pos != 0 {
                    state.pos -= 1;

                    // Echo the entire sequence
                    uart_putc!(r.USER_UART, 0x1b);
                    uart_putc!(r.USER_UART, '[');
                    uart_putc!(r.USER_UART, c);
                }
            }
            // Right arrow
            else if c == 'C' as u8 {
                if state.pos != state.len {
                    state.pos += 1;

                    // Echo the entire sequence
                    uart_putc!(r.USER_UART, 0x1b);
                    uart_putc!(r.USER_UART, '[');
                    uart_putc!(r.USER_UART, c);
                }
            } else {
                // Got ESC[-x which we don't understand. Just ignore it
            }
            state.escape_fsm = LineEditEscapeFSM::NONE;
        },
    }
}

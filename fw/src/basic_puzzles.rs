// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use rtfm::{Resource, Threshold};

use ::{msleep, hwrng_gen_pin};
use cli::*;

pub fn level0(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut success = false;

    r.USER_UART.claim(t, |user_uart, _t| {
        // Disable the UART RX interrupt. This disables the line editing
        // and allows us to directly read characters
        user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

        uart_putstr(user_uart,
            "Entering Level 0 - \"It's just like in the movies!\"\r\n\r\n\
            You have found a safe. It has one of those newfangled \"IoT\" locks.\r\n");

        let mut continue_running = true;
        loop {
            uart_putstr(user_uart, "Enter PIN: ");

            for i in 0..r.PUZZLES_STATE.level0_pin.len() {
                // Manually get a character
                let c = uart_getc_manual(user_uart);

                if c == 0x03 {
                    // Ctrl-C
                    uart_putstr(user_uart, "\r\n");
                    continue_running = false;
                    break;
                }

                uart_putc!(user_uart, c);

                let cint = match c as char {
                    '0'...'9' => {
                        c - '0' as u8
                    },
                    _ => {
                        // Dummy
                        10
                    }
                };

                if cint != r.PUZZLES_STATE.level0_pin[i] {
                    uart_putstr(user_uart, "\r\nIncorrect PIN!\r\n");
                    break;
                }

                // Digit is valid here
                if i == r.PUZZLES_STATE.level0_pin.len() - 1 {
                    uart_putstr(user_uart, "\r\nSUCCESS!\r\n");
                    success = true;
                    continue_running = false;
                }
            }

            if !continue_running {
                break;
            }
        }

        // Need to reenable interrupt
        user_uart.cr1.modify(|_, w| w.rxneie().bit(true));
    });

    if success {
        r.PUZZLES_STATE.last_cleared_level = 0;
        r.PUZZLES_STATE.cleared_levels |= 1 << 0;
    }

    (success, success)
}

pub fn level1(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut success = false;

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 1 - \"Let's get serious\"\r\n\r\n\
            Inside the safe was a smaller safe. Apparently the previous one was just a test.\r\n\
            Enter PIN: ");
    });

    loop {
        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let entered_pin = uart_get_line_blocking(rx_c, tx_p, || {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Malformed PIN!\r\nEnter PIN: ");
            });
        });
        let entered_pin = entered_pin.as_bytes();

        // User input a string now
        if entered_pin.len() == 0 {
            // Give up
            break;
        }

        if entered_pin.len() != r.PUZZLES_STATE.level1_pin.len() {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Incorrect PIN!\r\nEnter PIN: ");
            });
            continue;
        }

        for i in 0..r.PUZZLES_STATE.level1_pin.len() {
            let c = entered_pin[i];

            let cint = match c as char {
                '0'...'9' => {
                    c - '0' as u8
                },
                _ => {
                    // Dummy
                    10
                }
            };

            if cint != r.PUZZLES_STATE.level1_pin[i] {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Incorrect PIN!\r\nEnter PIN: ");
                });
                break;
            }

            // Digit is valid here
            if i == r.PUZZLES_STATE.level1_pin.len() - 1 {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "SUCCESS!\r\n");
                });
                success = true;
            }
        }

        if success {
            break;
        }
    }

    if success {
        r.PUZZLES_STATE.last_cleared_level = 1;
        r.PUZZLES_STATE.cleared_levels |= 1 << 1;
    }

    (success, success)
}

pub fn level2(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut success = false;

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 2 - \"Stop bruteforcing me!\"\r\n\r\n\
            While you were working on the safe, it received an \"OTA firmware update.\"\r\n\
            Enter PIN: ");
    });

    loop {
        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let entered_pin = uart_get_line_blocking(rx_c, tx_p, || {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Malformed PIN!\r\nEnter PIN: ");
            });
        });
        let entered_pin = entered_pin.as_bytes();

        // User input a string now
        if entered_pin.len() == 0 {
            // Give up
            break;
        }

        if entered_pin.len() == r.PUZZLES_STATE.level2_pin.len() {
            for i in 0..r.PUZZLES_STATE.level2_pin.len() {
                let c = entered_pin[i];

                let cint = match c as char {
                    '0'...'9' => {
                        c - '0' as u8
                    },
                    _ => {
                        // Dummy
                        10
                    }
                };

                if cint != r.PUZZLES_STATE.level2_pin[i] {
                    break;
                }

                // Digit is valid here
                if i == r.PUZZLES_STATE.level2_pin.len() - 1 {
                    success = true;
                }
            }
        }

        if success {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "SUCCESS!\r\n");
            });
            break;
        } else {
            // The pin is incorrect
            msleep(r.IDLE_MS_COUNTER, 5000);   // Deliberately exploitable timing attack

            r.PUZZLES_STATE.level2_tries_left -= 1;
            if r.PUZZLES_STATE.level2_tries_left > 0 {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Incorrect PIN!\r\nEnter PIN: ");
                });
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "GAME OVER!\r\n");
                });
                // Generate a new PIN for this level
                r.PUZZLES_STATE.level2_pin = hwrng_gen_pin(r.RNG, r.RNG_LAST_VAL);
                r.PUZZLES_STATE.level2_tries_left = 3;
                return (true, false);
            }
        }
    }

    if success {
        r.PUZZLES_STATE.last_cleared_level = 2;
        r.PUZZLES_STATE.cleared_levels |= 1 << 2;
    }

    (success, success)
}

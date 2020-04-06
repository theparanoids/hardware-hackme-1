// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use rtfm::{Resource, Threshold};
use core::str;

use ::msleep;
use cli::*;
use cpld::*;

const CPLD_16BIT_BITSTREAM: &'static [u8] = include_bytes!("match-16bit.bin");
const CPLD_64BIT_BITSTREAM: &'static [u8] = include_bytes!("match-64bit.bin");

pub fn level11(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart, "Programming CPLD, please wait... ");
    });

    cpld_erase(r.GPIOB, r.IDLE_MS_COUNTER);
    cpld_write_eeprom(r.GPIOB, r.IDLE_MS_COUNTER, |addr, biti| {
        let byte = CPLD_16BIT_BITSTREAM[(33 * addr + biti / 8) as usize];
        byte & (1 << (biti % 8)) != 0
    });

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart, "OK\r\n");
    });

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 11 - \"Secure Enclave\"\r\n\r\n\
            You have a device that requires a 4-digit PIN. The device claims\r\n\
            to use a \"secure\" enclave to verify this PIN. Can you break the pin?\r\n");
    });

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Enter PIN: ");
        });

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let mut entered_pin = [0u8; 4];
        loop {
            let pin_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nEnter PIN: ");
                });
            });

            if pin_str.len() == 0 {
                return (false, false);
            }

            if pin_str.len() == 4 {
                let mut problem = false;
                for (i, c) in pin_str.bytes().enumerate() {
                    match c as char {
                        '0'...'9' => {
                            entered_pin[i] = c - '0' as u8;
                        },
                        _ => {
                            problem = true;
                        }
                    }
                }

                if !problem {
                    break;
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Malformed input!\r\nEnter PIN: ");
                    });
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nEnter PIN: ");
                });
            }
        }

        let byte0: u8 = entered_pin[0] | (entered_pin[1] << 4);
        let byte1: u8 = entered_pin[2] | (entered_pin[3] << 4);

        while uart_getc_nowait(r.CPLD_UART).is_some() {}
        uart_putc!(r.CPLD_UART, byte0);
        msleep(r.IDLE_MS_COUNTER, 1);
        uart_putc!(r.CPLD_UART, byte1);
        msleep(r.IDLE_MS_COUNTER, 1000); // Deliberately long to make MITM injection easy

        let cin = uart_getc_nowait(r.CPLD_UART);

        if cin.is_none() {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "ERROR! \"Secure\" enclave not responding!\r\n");
            });
            continue;
        }
        let cin = cin.unwrap();

        if cin == 'N' as u8 {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Incorrect PIN!\r\n");
            });
            // Deliberate annoying delay
            msleep(r.IDLE_MS_COUNTER, 5000);
            continue;
        }

        // CPLD reply is yes, are we MITM or are we legit?
        if entered_pin == r.PUZZLES_STATE_LORGE.cpld_16bit_answer {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "SUCCESS!\r\n");
            });
        } else {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You cheater! You pass anyways.\r\n");
            });
        }
        r.PUZZLES_STATE.last_cleared_level = 11;
        r.PUZZLES_STATE.cleared_levels |= 1 << 11;
        return (true, true);
    }
}

pub fn level12(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart, "Programming CPLD, please wait... ");
    });

    cpld_erase(r.GPIOB, r.IDLE_MS_COUNTER);
    cpld_write_eeprom(r.GPIOB, r.IDLE_MS_COUNTER, |addr, biti| {
        let byte = CPLD_64BIT_BITSTREAM[(33 * addr + biti / 8) as usize];
        byte & (1 << (biti % 8)) != 0
    });

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart, "OK\r\n");
    });

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 12 - \"Secure Enclave, redux\"\r\n\r\n\
            The device has upgraded the secure enclave PIN to a 64-bit key.\r\n\
            This should be completely infeasible to brute-force, but maybe there\r\n\
            are other implementation bugs?\r\n\r\n\
            NOTE: The debugger firmware has been pre-modified to allow you to\r\n\
            intercept data going from the CPLD to the microcontroller.\r\n\
            This will almost certainly be of use to you.\r\n");
    });

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Enter key: ");
        });

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let mut entered_key = [0u8; 16];
        loop {
            let key_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nEnter key: ");
                });
            });

            if key_str.len() == 0 {
                return (false, false);
            }

            if key_str.len() == 16 {
                let mut problem = false;
                for (i, c) in key_str.bytes().enumerate() {
                    match c as char {
                        '0'...'9' => {
                            entered_key[i] = c - '0' as u8;
                        },
                        'A'...'F' => {
                            entered_key[i] = c - 'A' as u8 + 10;
                        },
                        'a'...'f' => {
                            entered_key[i] = c - 'a' as u8 + 10;
                        },
                        _ => {
                            problem = true;
                        }
                    }
                }

                if !problem {
                    break;
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Malformed input!\r\nEnter key: ");
                    });
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nEnter key: ");
                });
            }
        }

        let keybytes: [u8; 8] = [
            (entered_key[0] << 4) | (entered_key[1]),
            (entered_key[2] << 4) | (entered_key[3]),
            (entered_key[4] << 4) | (entered_key[5]),
            (entered_key[6] << 4) | (entered_key[7]),
            (entered_key[8] << 4) | (entered_key[9]),
            (entered_key[10] << 4) | (entered_key[11]),
            (entered_key[12] << 4) | (entered_key[13]),
            (entered_key[14] << 4) | (entered_key[15]),
        ];

        while uart_getc_nowait(r.CPLD_UART).is_some() {}

        // Deliberately long to make MITM injection easy
        // Note that this is _after_ we already clear out leftover bytes
        msleep(r.IDLE_MS_COUNTER, 2000);

        for i in 0..8 {
            uart_putc!(r.CPLD_UART, keybytes[i]);
            msleep(r.IDLE_MS_COUNTER, 1);
        }

        // A second deliberately long to make MITM injection easy
        msleep(r.IDLE_MS_COUNTER, 2000);

        let cin = uart_getc_nowait(r.CPLD_UART);

        if cin.is_none() {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "ERROR! \"Secure\" enclave not responding!\r\n");
            });
            continue;
        }
        let cin = cin.unwrap();

        if cin == 'N' as u8 {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Incorrect key!\r\n");
            });
            // Deliberate annoying delay
            msleep(r.IDLE_MS_COUNTER, 5000);
            continue;
        }

        // CPLD reply is yes
        uart.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "SUCCESS!\r\n");
        });
        r.PUZZLES_STATE.last_cleared_level = 12;
        r.PUZZLES_STATE.cleared_levels |= 1 << 12;
        return (true, true);
    }
}

pub fn level13(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 13 - \"CPLD reverse engineering?\"\r\n\r\n\
            We really would like to extract the secret key from the CPLD.\r\n\
            Maybe it is possible to understand and reverse-engineer the\r\n\
            bitstream programmed into the device? The vendors insist that\r\n\
            this cannot be done, but are they right?\r\n\r\n\
            As a warm-up, we have hidden 64 bits somewhere inside the CPLD\r\n\
            in unused configuration bits. Find them.\r\n\r\n");
    });

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart,
                "Do you want to:\r\n\
                1) Dump the CPLD bitstream\r\n\
                2) Enter the extracted bits\r\n\
                Enter choice: ");
        });

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);

        let choice_str = uart_get_line_blocking(rx_c, tx_p, || {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Malformed input!\r\nEnter choice: ");
            });
        });

        if choice_str.len() == 0 {
            return (false, false);
        }

        match choice_str.as_ref() {
            "1" => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart,
                        "// DEVICE XC2C32A-6-VQ44\r\n\
                        \r\n\
                        // This is a dump of the CPLD programming information in _physical_ bit order.\r\n\
                        // If you have ever seen a .jed file, those files are in a shuffled _logical_\r\n\
                        // bit order. If you count the bits, you might notice a discrepancy between the\r\n\
                        // number of physical bits and the number of logical bits. This is your hint.\r\n\r\n");

                    for addr in 0..49 {
                        for biti in 0..260 {
                            let byte = CPLD_64BIT_BITSTREAM[(33 * addr + biti / 8) as usize];
                            let bit = byte & (1 << (biti % 8)) != 0;

                            if bit {
                                uart_putc!(user_uart, '1');
                            } else {
                                uart_putc!(user_uart, '0');
                            }
                        }
                        uart_putstr(user_uart, "\r\n");
                    }

                    // USERCODE row (dummy)
                    for _ in 0..260 {
                        uart_putc!(user_uart, '0');
                    }
                    uart_putstr(user_uart, "\r\n");
                });
            },
            "2" => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Enter hidden bytes (in hex): ");
                });

                let mut entered_key = [0u8; 16];
                loop {
                    let key_str = uart_get_line_blocking(rx_c, tx_p, || {
                        uart.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "Malformed input!\r\nEnter hidden bytes (in hex): ");
                        });
                    });

                    if key_str.len() == 0 {
                        return (false, false);
                    }

                    if key_str.len() == 16 {
                        let mut problem = false;
                        for (i, c) in key_str.bytes().enumerate() {
                            match c as char {
                                '0'...'9' => {
                                    entered_key[i] = c - '0' as u8;
                                },
                                'A'...'F' => {
                                    entered_key[i] = c - 'A' as u8 + 10;
                                },
                                'a'...'f' => {
                                    entered_key[i] = c - 'a' as u8 + 10;
                                },
                                _ => {
                                    problem = true;
                                }
                            }
                        }

                        if !problem {
                            break;
                        } else {
                            uart.claim(t, |user_uart, _t| {
                                uart_putstr(user_uart, "Malformed input!\r\nEnter hidden bytes (in hex): ");
                            });
                        }
                    } else {
                        uart.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "Malformed input!\r\nEnter hidden bytes (in hex): ");
                        });
                    }
                }

                let keybytes: [u8; 8] = [
                    (entered_key[0] << 4) | (entered_key[1]),
                    (entered_key[2] << 4) | (entered_key[3]),
                    (entered_key[4] << 4) | (entered_key[5]),
                    (entered_key[6] << 4) | (entered_key[7]),
                    (entered_key[8] << 4) | (entered_key[9]),
                    (entered_key[10] << 4) | (entered_key[11]),
                    (entered_key[12] << 4) | (entered_key[13]),
                    (entered_key[14] << 4) | (entered_key[15]),
                ];

                if keybytes == r.PUZZLES_STATE_LORGE.cpld_center_hidden {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "SUCCESS!\r\n");
                    });
                    r.PUZZLES_STATE.last_cleared_level = 13;
                    r.PUZZLES_STATE.cleared_levels |= 1 << 13;
                    return (true, true);
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "WRONG!\r\n");
                    });
                }
            },
            _ => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Illegal choice!\r\n");
                });
            }
        }
    }
}

pub fn level14(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 14 - \"CPLD reverse engineering! Part 2\"\r\n\r\n\
            We really would like to extract the secret key from the CPLD.\r\n\
            Maybe it is possible to understand and reverse-engineer the\r\n\
            bitstream programmed into the device? The vendors insist that\r\n\
            this cannot be done, but are they right?\r\n\r\n\
            CPLDs are designed to implement logic equations in sum-of-products\r\n\
            form. This means that there are a set of AND gates that connect to\r\n\
            OR gates. Usually not all of the AND gates are used. This design\r\n\
            seems to have some \"unusual\" AND gates. What is going on?\r\n\r\n");
    });

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart,
                "Do you want to:\r\n\
                1) Dump the CPLD bitstream\r\n\
                2) Enter the secret data\r\n\
                Enter choice: ");
        });

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);

        let choice_str = uart_get_line_blocking(rx_c, tx_p, || {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Malformed input!\r\nEnter choice: ");
            });
        });

        if choice_str.len() == 0 {
            return (false, false);
        }

        match choice_str.as_ref() {
            "1" => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "// DEVICE XC2C32A-6-VQ44\r\n\r\n");

                    for addr in 0..49 {
                        for biti in 0..260 {
                            let byte = CPLD_64BIT_BITSTREAM[(33 * addr + biti / 8) as usize];
                            let bit = byte & (1 << (biti % 8)) != 0;

                            if bit {
                                uart_putc!(user_uart, '1');
                            } else {
                                uart_putc!(user_uart, '0');
                            }
                        }
                        uart_putstr(user_uart, "\r\n");
                    }

                    // USERCODE row (dummy)
                    for _ in 0..260 {
                        uart_putc!(user_uart, '0');
                    }
                    uart_putstr(user_uart, "\r\n");
                });
            },
            "2" => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Enter secret data (ASCII text): ");
                });

                let key_str = uart_get_line_blocking(rx_c, tx_p, || {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Malformed input!\r\nEnter secret data (ASCII text): ");
                    });
                });

                if key_str.len() == 0 {
                    return (false, false);
                }

                if key_str.trim() == str::from_utf8(&r.PUZZLES_STATE_LORGE.cpld_pterm_hidden).unwrap() {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "SUCCESS!\r\n");
                    });
                    r.PUZZLES_STATE.last_cleared_level = 14;
                    r.PUZZLES_STATE.cleared_levels |= 1 << 14;
                    return (true, true);
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "WRONG!\r\n");
                    });
                }
            },
            _ => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Illegal choice!\r\n");
                });
            }
        }
    }
}

pub fn level15(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 15 - \"CPLD reverse engineering - the final puzzle\"\r\n\r\n\
            We really would like to extract the secret key from the CPLD.\r\n\
            Maybe it is possible to understand and reverse-engineer the\r\n\
            bitstream programmed into the device? The vendors insist that\r\n\
            this cannot be done, but are they right?\r\n\r\n\
            Your final goal is to actually recover the secret 64-bit key recognized\r\n\
            by the CPLD. Good luck!\r\n\r\n");
    });

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Enter key: ");
        });

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let mut entered_key = [0u8; 16];
        loop {
            let key_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nEnter key: ");
                });
            });

            if key_str.len() == 0 {
                return (false, false);
            }

            if key_str.len() == 16 {
                let mut problem = false;
                for (i, c) in key_str.bytes().enumerate() {
                    match c as char {
                        '0'...'9' => {
                            entered_key[i] = c - '0' as u8;
                        },
                        'A'...'F' => {
                            entered_key[i] = c - 'A' as u8 + 10;
                        },
                        'a'...'f' => {
                            entered_key[i] = c - 'a' as u8 + 10;
                        },
                        _ => {
                            problem = true;
                        }
                    }
                }

                if !problem {
                    break;
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Malformed input!\r\nEnter key: ");
                    });
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nEnter key: ");
                });
            }
        }

        let keybytes: [u8; 8] = [
            (entered_key[0] << 4) | (entered_key[1]),
            (entered_key[2] << 4) | (entered_key[3]),
            (entered_key[4] << 4) | (entered_key[5]),
            (entered_key[6] << 4) | (entered_key[7]),
            (entered_key[8] << 4) | (entered_key[9]),
            (entered_key[10] << 4) | (entered_key[11]),
            (entered_key[12] << 4) | (entered_key[13]),
            (entered_key[14] << 4) | (entered_key[15]),
        ];

        if keybytes == r.PUZZLES_STATE_LORGE.cpld_64bit_answer {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "SUCCESS!\r\n");
            });
            r.PUZZLES_STATE.last_cleared_level = 15;
            r.PUZZLES_STATE.cleared_levels |= 1 << 15;
            return (true, true);
        } else {
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "WRONG!\r\n");
            });
        }
    }
}

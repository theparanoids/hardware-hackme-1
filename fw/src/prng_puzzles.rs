// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use alloc::string::ToString;
use alloc::borrow::ToOwned;
use core::str;
use rtfm::{Resource, Threshold};

use ::hwrng_get_u32;
use except_and_svc::invoke_user_code;
use cli::*;
use crypto::*;

pub fn level3(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut prng = ChaCha20PRNGState::new(r.RNG, r.RNG_LAST_VAL);

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 3 - \"Let's play a game\"\r\n\r\n\
            After finally opening the safe, you find... a dice game?\r\n");
    });

    let mut dollars: u32 = 100;

    while dollars < 100000 {
        if dollars == 0 {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart,
                    "Sorry, but you are out of money. Better luck next time.\r\n");
            });
            return (false, false);
        }

        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "You currently have $");
            uart_putstr(user_uart, &dollars.to_string());
            uart_putstr(user_uart, ". You need $100000 to proceed.\r\n\
                Place a bet and roll the dice. If you guess correctly, you get double back.\r\n\
                Bet? ");
        });

        let dice_actual;
        loop {
            let prngval = prng.getu8();
            if prngval < 252 {
                // This is needed for de-biasing
                dice_actual = (prngval % 6) as u32;
                break;
            }
        }

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let bet;
        loop {
            let bet_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            });

            if bet_str.len() == 0 {
                return (false, false);
            }

            if let Ok(bet_parsed) = bet_str.parse::<u32>() {
                // Deliberately broken check
                if (bet_parsed as i32) > (dollars as i32) {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "You cannot bet more than you have!\r\nBet? ");
                    });
                } else if bet_parsed == 0 {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Nice try. Bet more than $0 please.\r\nBet? ");
                    });
                } else {
                    bet = bet_parsed;
                    break;
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            }
        }

        uart.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Guess for dice roll (1-6)? ");
        });

        let dice_guess;
        loop {
            let dice_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                });
            });

            if dice_str.len() == 0 {
                return (false, false);
            }

            if let Ok(guess_parsed) = dice_str.parse::<u32>() {
                if guess_parsed >= 1 && guess_parsed <= 6 {
                    dice_guess = guess_parsed - 1;
                    break;
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Illegal guess!\r\nGuess for dice roll (1-6)? ");
                    });
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                });
            }
        }

        if dice_guess == dice_actual {
            // Give you money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed correctly!\r\n");
            });
            dollars = dollars.saturating_add(bet);
        } else {
            // Take away money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed incorrectly :(\r\n\
                    The dice roll was a ");
                uart_putstr(user_uart, &(dice_actual + 1).to_string());
                uart_putstr(user_uart, "\r\n");
            });
            // Wrapping on purpose
            dollars = dollars.wrapping_sub(bet);
        }
    }

    // If we got here, the user won the game
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Congratulations! You win!\r\n");
    });
    r.PUZZLES_STATE.last_cleared_level = 3;
    r.PUZZLES_STATE.cleared_levels |= 1 << 3;
    (true, true)
}

// XXX TODO: How to avoid copy-paste?
pub fn level4(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut prng = ChaCha20PRNGState::broken(&r.PUZZLES_STATE_LORGE.level4_seed);

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 4 - \"Play by the rules\"\r\n\r\n\
            The same dice game, but bugfixed\r\n");
    });

    let mut dollars: u32 = 100;

    while dollars < 100000 {
        if dollars == 0 {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart,
                    "Sorry, but you are out of money. Better luck next time.\r\n");
            });
            return (false, false);
        }

        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "You currently have $");
            uart_putstr(user_uart, &dollars.to_string());
            uart_putstr(user_uart, ". You need $100000 to proceed.\r\n\
                Place a bet and roll the dice. If you guess correctly, you get double back.\r\n\
                Bet? ");
        });

        let dice_actual;
        loop {
            let prngval = prng.getu8();
            if prngval < 252 {
                // This is needed for de-biasing
                dice_actual = (prngval % 6) as u32;
                break;
            }
        }

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let bet;
        loop {
            let bet_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            });

            if bet_str.len() == 0 {
                return (false, false);
            }

            if let Ok(bet_parsed) = bet_str.parse::<u32>() {
                if bet_parsed > dollars {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "You cannot bet more than you have!\r\nBet? ");
                    });
                } else if bet_parsed == 0 {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Nice try. Bet more than $0 please.\r\nBet? ");
                    });
                } else {
                    bet = bet_parsed;
                    break;
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            }
        }

        uart.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Guess for dice roll (1-6)? ");
        });

        let dice_guess;
        loop {
            let dice_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                });
            });

            if dice_str.len() == 0 {
                return (false, false);
            }

            if let Ok(guess_parsed) = dice_str.parse::<u32>() {
                if guess_parsed >= 1 && guess_parsed <= 6 {
                    dice_guess = guess_parsed - 1;
                    break;
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Illegal guess!\r\nGuess for dice roll (1-6)? ");
                    });
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                });
            }
        }

        if dice_guess == dice_actual {
            // Give you money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed correctly!\r\n");
            });
            dollars = dollars.saturating_add(bet);
        } else {
            // Take away money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed incorrectly :(\r\n\
                    The dice roll was a ");
                uart_putstr(user_uart, &(dice_actual + 1).to_string());
                uart_putstr(user_uart, "\r\n");
            });
            dollars = dollars.saturating_sub(bet);
        }
    }

    // If we got here, the user won the game
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Congratulations! You win!\r\n");
    });
    r.PUZZLES_STATE.last_cleared_level = 4;
    r.PUZZLES_STATE.cleared_levels |= 1 << 4;
    (true, true)
}

// XXX TODO: How to avoid copy-paste?
pub fn level5(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut prng = LCGPRNGState::new(hwrng_get_u32(r.RNG, r.RNG_LAST_VAL));

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 5 - \"Cosmic rays\"\r\n\r\n\
            The dice game has been upgraded with a new special proprietary\r\n\
            true-random-number-generator. We've heard it involves something about \"linear.\"\r\n");
    });

    let mut dollars: u32 = 100;

    while dollars < 100000 {
        if dollars == 0 {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart,
                    "Sorry, but you are out of money. Better luck next time.\r\n");
            });
            return (false, false);
        }

        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "You currently have $");
            uart_putstr(user_uart, &dollars.to_string());
            uart_putstr(user_uart, ". You need $100000 to proceed.\r\n\
                Place a bet and roll the dice. If you guess correctly, you get double back.\r\n\
                Bet? ");
        });

        let dice_actual;
        loop {
            let prngval = prng.getu8();
            if prngval < 252 {
                // This is needed for de-biasing
                dice_actual = (prngval % 6) as u32;
                break;
            }
        }

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let bet;
        loop {
            let bet_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            });

            if bet_str.len() == 0 {
                return (false, false);
            }

            if let Ok(bet_parsed) = bet_str.parse::<u32>() {
                if bet_parsed > dollars {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "You cannot bet more than you have!\r\nBet? ");
                    });
                } else if bet_parsed == 0 {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Nice try. Bet more than $0 please.\r\nBet? ");
                    });
                } else {
                    bet = bet_parsed;
                    break;
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            }
        }

        uart.claim(t, |user_uart, _t| {
            // // DEBUG DEBUG DEBUG
            // uart_putu8(user_uart, dice_actual as u8);
            uart_putstr(user_uart, "Guess for dice roll (1-6)? ");
        });

        let dice_guess;
        loop {
            let dice_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                });
            });

            if dice_str.len() == 0 {
                return (false, false);
            }

            if let Ok(guess_parsed) = dice_str.parse::<u32>() {
                if guess_parsed >= 1 && guess_parsed <= 6 {
                    dice_guess = guess_parsed - 1;
                    break;
                } else {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Illegal guess!\r\nGuess for dice roll (1-6)? ");
                    });
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                });
            }
        }

        if dice_guess == dice_actual {
            // Give you money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed correctly!\r\n");
            });
            dollars = dollars.saturating_add(bet);
        } else {
            // Take away money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed incorrectly :(\r\n\
                    The dice roll was a ");
                uart_putstr(user_uart, &(dice_actual + 1).to_string());
                uart_putstr(user_uart, "\r\n");
            });
            dollars = dollars.saturating_sub(bet);
        }
    }

    // If we got here, the user won the game
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Congratulations! You win!\r\n");
    });
    r.PUZZLES_STATE.last_cleared_level = 5;
    r.PUZZLES_STATE.cleared_levels |= 1 << 5;
    (true, true)
}

// XXX TODO: How to avoid copy-paste?
pub fn level6(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    let mut prng = ChaCha20PRNGState::new(r.RNG, r.RNG_LAST_VAL);

    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 6 - \"Consultants fixed our RNG\"\r\n\r\n\
            The same dice game, but completely rewritten PRNG and *input validation*\r\n");
    });

    let mut dollars: u32 = 100;

    while dollars < 100000 {
        if dollars == 0 {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart,
                    "Sorry, but you are out of money. Better luck next time.\r\n");
            });
            return (false, false);
        }

        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "You currently have $");
            uart_putstr(user_uart, &dollars.to_string());
            uart_putstr(user_uart, ". You need $100000 to proceed.\r\n\
                Place a bet and roll the dice. If you guess correctly, you get double back.\r\n\
                Bet? ");
        });

        let dice_actual;
        loop {
            let prngval = prng.getu8();
            if prngval < 252 {
                // This is needed for de-biasing
                dice_actual = (prngval % 6) as u32;
                break;
            }
        }

        let (rx_c, tx_p, uart) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART);
        let bet;
        loop {
            let bet_str = uart_get_line_blocking(rx_c, tx_p, || {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            });

            if bet_str.len() == 0 {
                return (false, false);
            }

            if let Ok(bet_parsed) = bet_str.parse::<u32>() {
                if bet_parsed > dollars {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "You cannot bet more than you have!\r\nBet? ");
                    });
                } else if bet_parsed == 0 {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "Nice try. Bet more than $0 please.\r\nBet? ");
                    });
                } else {
                    bet = bet_parsed;
                    break;
                }
            } else {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Malformed input!\r\nBet? ");
                });
            }
        }

        uart.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Guess for dice roll (1-6)? ");
        });

        let dice_guess;
        loop {
            if let Some(cmdline) = rx_c.dequeue() {
                if cmdline.len == 0 {
                    tx_p.enqueue(cmdline).unwrap();
                    return (false, false);
                }

                // FAKE buffer overflow here
                if cmdline.len > 80 {
                    unsafe {
                        // Fill everything else with NOPs
                        for i in 0..((0x20000 - 0x8000) / 4) {
                            *((0x20008000 + i * 4) as *mut u32) = 0x46c046c0;   // nop, nop
                        }

                        // Copy a jump opcode to 0x20009000 at 0x20008000
                        *(0x20008000 as *mut u32) = 0xbffef000;                 // b.w +0x1000

                        // Copy the PRNG state to 0x20008100
                        *(0x200080E0 as *mut u32) = 0x43616843;     // "ChaC
                        *(0x200080E4 as *mut u32) = 0x30326168;     //  ha20
                        *(0x200080E8 as *mut u32) = 0x4e525020;     //   PRN
                        *(0x200080EC as *mut u32) = 0x74732047;     //  G st
                        *(0x200080F0 as *mut u32) = 0x20657461;     //  ate 
                        *(0x200080F4 as *mut u32) = 0x68207369;     //  is h
                        *(0x200080F8 as *mut u32) = 0x20657265;     //  ere 
                        *(0x200080FC as *mut u32) = 0x3e2d2d2d;     //  --->"
                        ::core::ptr::copy_nonoverlapping(&prng, 0x20008100 as *mut ChaCha20PRNGState, 1);
                        debug_assert!(::core::mem::size_of::<ChaCha20PRNGState>() % 4 == 0);

                        // Copy the user code to 0x20009000
                        ::core::ptr::copy_nonoverlapping(&cmdline.buf[0], 0x20009000 as *mut u8, cmdline.len);
                        if cmdline.len % 4 != 0 {
                            for i in 0..(4 - (cmdline.len % 4)) {
                                *((0x20009000 + cmdline.len + i) as *mut u8) = 0;
                            }
                        }

                        // Done with the cmdline
                        tx_p.enqueue(cmdline).unwrap();

                        // Read the address to jump to from 0x20009000 + 80
                        let jump_to_address = *((0x20009000 + 80) as *const u32);

                        // Copy a return trampoline to 0x20010000
                        *(0x20010000 as *mut u32) = 0xdf002000;     // movs r0, #0; svc 0

                        // Now actually run the user code and make their return value the dice guess
                        dice_guess = invoke_user_code(jump_to_address, 0x20010001);
                        break;
                    }
                } else {
                    let cmdline_str = if let Ok(cmdline) = str::from_utf8(&cmdline.buf[0..cmdline.len]) {
                        Some(cmdline.to_owned())
                    } else {
                        None
                    };

                    // Done with the cmdline
                    tx_p.enqueue(cmdline).unwrap();

                    if let Some(cmdline) = cmdline_str {
                        if let Ok(guess_parsed) = cmdline.parse::<u32>() {
                            if guess_parsed >= 1 && guess_parsed <= 6 {
                                dice_guess = guess_parsed - 1;
                                break;
                            } else {
                                uart.claim(t, |user_uart, _t| {
                                    uart_putstr(user_uart, "Illegal guess!\r\nGuess for dice roll (1-6)? ");
                                });
                            }
                        } else {
                            uart.claim(t, |user_uart, _t| {
                                uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                            });
                        }
                    } else {
                        uart.claim(t, |user_uart, _t| {
                            uart_putstr(user_uart, "Malformed input!\r\nGuess for dice roll (1-6)? ");
                        });
                    }
                }

            } else {
                ::rtfm::wfi();
            }
        }

        if dice_guess == dice_actual {
            // Give you money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed correctly!\r\n");
            });
            dollars = dollars.saturating_add(bet);
        } else {
            // Take away money
            uart.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "You guessed incorrectly :(\r\n\
                    The dice roll was a ");
                uart_putstr(user_uart, &(dice_actual + 1).to_string());
                uart_putstr(user_uart, "\r\n");
            });
            dollars = dollars.saturating_sub(bet);
        }
    }

    // If we got here, the user won the game
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Congratulations! You win!\r\n");
    });
    r.PUZZLES_STATE.last_cleared_level = 6;
    r.PUZZLES_STATE.cleared_levels |= 1 << 6;
    (true, true)
}

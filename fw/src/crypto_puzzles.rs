// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use rtfm::{Resource, Threshold};
use core::str;
use sha2::{Sha256, Digest};

use ::{LEVEL9_CLEAR_FLAG, LEVEL10_CLEAR_FLAG};
use aes_nostd;
use cryptastic;
use cli::*;
use except_and_svc::invoke_user_code;

use aes_nostd::symmetriccipher::BlockDecryptor;

fn strncmp_like(a: &[u8], b: &[u8]) -> bool {
    assert!(a.len() == b.len());

    for i in 0..a.len() {
        if a[i] == 0 && b[i] == 0 {
            return true;
        }

        if a[i] != b[i] {
            return false;
        }
    }

    true
}

pub fn get_device_id_hex() -> [u8; 24] {
    let mut deviceid = [0u8; 12];
    unsafe { ::core::ptr::copy_nonoverlapping(0x1FFF7A10 as *const u8, &mut deviceid[0], 12); }

    let mut deviceid_binary = [0u8; 24];
    for i in 0..12 {
        deviceid_binary[i * 2 + 0] = HEXLUT[(deviceid[i] >> 4) as usize];
        deviceid_binary[i * 2 + 1] = HEXLUT[(deviceid[i] & 0xF) as usize];
    }
    deviceid_binary
}

fn validate_config_file(cfg_bytes: &[u8]) -> (bool, bool) {
    let mut locked_state = true;
    let mut matched_devid = false;

    if let Ok(cfgfile) = str::from_utf8(cfg_bytes) {
        for line in cfgfile.lines() {
            let line = line.trim();

            if line.starts_with("DEVICE_ID=") {
                let (_, devid) = line.split_at(10);
                let actual_devid = get_device_id_hex();

                if devid.as_bytes() == actual_devid {
                    matched_devid = true;
                }
            } else if line.starts_with("LOCKED_STATE=") {
                let (_, lockstate_str) = line.split_at(13);

                if lockstate_str == "false" {
                    locked_state = false;
                }
            }
        }
    }

    (locked_state, matched_devid)
}

pub fn level7(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 7 - \"No twiizers required\"\r\n\r\n\
            ACME Corporation has just designed a consumer electronics product.\r\n\
            You wonder how secure it is. RSA-2048 should be unbreakable, right?\r\n\r\n\
            You extract the following configuration data blob from flash:\r\n");

        for i in 0..r.PUZZLES_STATE_LORGE.level7_example.len() {
            uart_putu8(user_uart, r.PUZZLES_STATE_LORGE.level7_example[i]);
            if i % 16 == 15 {
                uart_putstr(user_uart, "\r\n");
            }
        }

        uart_putstr(user_uart, "\r\nPerhaps you can still modify this somehow? (Upload 384 bytes now) \r\n");
    });

    let upload_buf = &mut r.LEVEL78_UPLOAD_BUF;
    r.USER_UART.claim(t, |user_uart, _t| {
        // Disable the UART RX interrupt. This disables the line editing
        // and allows us to directly read characters
        user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

        for i in 0..upload_buf.len() {
            upload_buf[i] = uart_getc_manual(user_uart);
        }

        // Need to reenable interrupt
        user_uart.cr1.modify(|_, w| w.rxneie().bit(true));
    });

    let mut hasher = Sha256::default();
    hasher.input(&upload_buf[256..384]);
    let hash = hasher.result();

    let sig = &mut r.WORK_BIGNUM_2048_4;
    for i in 0..64 {
        let x =
            ((upload_buf[(63 - i) * 4 + 0] as u32) << 24) |
            ((upload_buf[(63 - i) * 4 + 1] as u32) << 16) |
            ((upload_buf[(63 - i) * 4 + 2] as u32) << 8) |
            ((upload_buf[(63 - i) * 4 + 3] as u32) << 0);
        sig.0[i] = x;
    }

    let rsa_op_result = &mut r.WORK_BIGNUM_2048_3;
    **rsa_op_result = cryptastic::rsa_pubkey_op(sig, r.LEVEL7_MODULUS, r.LEVEL7_R, r.LEVEL7_RR, 65537,
        r.WORK_BIGNUM_4096O, r.WORK_BIGNUM_2048O, r.WORK_BIGNUM_2048, r.WORK_BIGNUM_2048_2);

    // Deliberate bug: No padding checking
    // Deliberate bug: strncmp

    let mut rsa_op_result_hash = [0u8; 32];
    for i in 0..32 {
        let x32 = rsa_op_result.0[7 - (i / 4)];
        let x8 = x32 >> (8 * (3 - (i % 4)));
        rsa_op_result_hash[i] = x8 as u8;
    }

    if strncmp_like(&hash, &rsa_op_result_hash) {
        let (locked_state, matched_devid) = validate_config_file(&upload_buf[256..384]);

        if !matched_devid {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Uploaded data has invalid DEVICE_ID!\r\n");
            });
        } else if locked_state {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Device still has LOCKED_STATE=true!\r\n");
            });
        } else {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "SUCCESS!\r\n");
            });
            r.PUZZLES_STATE.last_cleared_level = 7;
            r.PUZZLES_STATE.cleared_levels |= 1 << 7;
            return (true, true);
        }
    } else {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Uploaded data has invalid signature!\r\n");
        });
    }

    (false, false)
}

pub fn level8(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 8 - \"not success overflow\"\r\n\r\n\
            ACME Corporation's competitor also has a consumer electronics product.\r\n\
            You wonder how secure it is. secp256r1 should still be unbreakable, right?\r\n\r\n\
            You extract the following configuration data blob from flash:\r\n");

        for i in 0..r.PUZZLES_STATE_LORGE.level8_example1.len() {
            uart_putu8(user_uart, r.PUZZLES_STATE_LORGE.level8_example1[i]);
            if i % 16 == 15 {
                uart_putstr(user_uart, "\r\n");
            }
        }

        uart_putstr(user_uart,
            "\r\nYou also have a friend who loans you another device.\r\n\
            You extract the following configuration data blob from this other device:\r\n");

        for i in 0..r.PUZZLES_STATE_LORGE.level8_example2.len() {
            uart_putu8(user_uart, r.PUZZLES_STATE_LORGE.level8_example2[i]);
            if i % 16 == 15 {
                uart_putstr(user_uart, "\r\n");
            }
        }

        uart_putstr(user_uart, "\r\nPerhaps you can still modify this somehow? (Upload 192 bytes now) \r\n");
    });

    let upload_buf = &mut r.LEVEL78_UPLOAD_BUF[..192];
    r.USER_UART.claim(t, |user_uart, _t| {
        // Disable the UART RX interrupt. This disables the line editing
        // and allows us to directly read characters
        user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

        for i in 0..upload_buf.len() {
            upload_buf[i] = uart_getc_manual(user_uart);
        }

        // Need to reenable interrupt
        user_uart.cr1.modify(|_, w| w.rxneie().bit(true));
    });

    let mut hasher = Sha256::default();
    hasher.input(&upload_buf[64..192]);
    let hasher_result = hasher.result();

    let hash = &mut r.WORK_BIGNUM_256_0;
    let sig_r = &mut r.WORK_BIGNUM_256_1;
    let sig_s = &mut r.WORK_BIGNUM_256_2;

    for i in 0..8 {
        let x =
            ((hasher_result[(7 - i) * 4 + 0] as u32) << 24) |
            ((hasher_result[(7 - i) * 4 + 1] as u32) << 16) |
            ((hasher_result[(7 - i) * 4 + 2] as u32) << 8) |
            ((hasher_result[(7 - i) * 4 + 3] as u32) << 0);
        hash.0[i] = x;
    }
    for i in 0..8 {
        let x =
            ((upload_buf[(7 - i) * 4 + 0] as u32) << 24) |
            ((upload_buf[(7 - i) * 4 + 1] as u32) << 16) |
            ((upload_buf[(7 - i) * 4 + 2] as u32) << 8) |
            ((upload_buf[(7 - i) * 4 + 3] as u32) << 0);
        sig_r.0[i] = x;
    }
    for i in 0..8 {
        let x =
            ((upload_buf[32 + (7 - i) * 4 + 0] as u32) << 24) |
            ((upload_buf[32 + (7 - i) * 4 + 1] as u32) << 16) |
            ((upload_buf[32 + (7 - i) * 4 + 2] as u32) << 8) |
            ((upload_buf[32 + (7 - i) * 4 + 3] as u32) << 0);
        sig_s.0[i] = x;
    }

    if cryptastic::ecdsa_secp256r1_verify_sig(r.LEVEL8_PUBK, r.LEVEL8_PUBK_PLUS_G, &hash, &sig_r, &sig_s) {
        let (locked_state, matched_devid) = validate_config_file(&upload_buf[64..192]);

        if !matched_devid {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Uploaded data has invalid DEVICE_ID!\r\n");
            });
        } else if locked_state {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "Device still has LOCKED_STATE=true!\r\n");
            });
        } else {
            r.USER_UART.claim(t, |user_uart, _t| {
                uart_putstr(user_uart, "SUCCESS!\r\n");
            });
            r.PUZZLES_STATE.last_cleared_level = 8;
            r.PUZZLES_STATE.cleared_levels |= 1 << 8;
            return (true, true);
        }
    } else {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart, "Uploaded data has invalid signature!\r\n");
        });
    }

    (false, false)
}

pub fn level9(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 9 - \"secure firmware loading\"\r\n\r\n\
            You are reverse-engineering a device with encrypted firmware.\r\n\
            The vendor tells you that the firmware is encrypted with an \"unbreakable\"\r\n\
            one-time-pad scheme.\r\n\r\n");
    });

    unsafe { LEVEL9_CLEAR_FLAG = false; }

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart,
                "Do you want to:\r\n\
                1) Download the encrypted code (outputs 8192 binary bytes)\r\n\
                2) Upload encrypted code (upload 8192 bytes)\r\n\
                Enter choice: ");
        });

        let (rx_c, tx_p, uart, puzzles_state_lorge) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART, &r.PUZZLES_STATE_LORGE);

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
                    for i in 0..puzzles_state_lorge.level9_code.len() {
                        uart_putc!(user_uart, puzzles_state_lorge.level9_code[i]);
                    }
                });
            },
            "2" => {
                // Fill everything else with NOPs
                unsafe {
                    for i in 0..((0x20000 - 0x8000) / 4) {
                        *((0x20008000 + i * 4) as *mut u32) = 0x46c046c0;   // nop, nop
                    }
                }

                let load_code_addr = 0x20008000 as *mut u8;
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Upload 8192 bytes now:");

                    // Disable the UART RX interrupt. This disables the line editing
                    // and allows us to directly read characters
                    user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

                    for i in 0..8192 {
                        unsafe {
                            *load_code_addr.offset(i) = uart_getc_manual(user_uart);
                        }
                    }

                    // Need to reenable interrupt
                    user_uart.cr1.modify(|_, w| w.rxneie().bit(true));

                    uart_putstr(user_uart, "\r\n");
                });

                // Now we XOR the uploaded code with the XOR mask
                for i in 0..r.PUZZLES_STATE_LORGE.level9_xor.len() {
                    unsafe {
                        *load_code_addr.offset(i as isize) ^= r.PUZZLES_STATE_LORGE.level9_xor[i];
                    }
                }

                // Jump to the uploaded code
                // Copy a return trampoline to 0x20010000
                unsafe {
                    *(0x20010000 as *mut u16) = 0xbe00;         // bkpt 0; stops "falling straight through"
                }
                // Now actually run the user code
                invoke_user_code(0x20008000, 0x20010001);

                if unsafe { LEVEL9_CLEAR_FLAG } {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "SUCCESS!\r\n");
                    });
                    r.PUZZLES_STATE.last_cleared_level = 9;
                    r.PUZZLES_STATE.cleared_levels |= 1 << 9;
                    return (true, true);
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

pub fn level10(t: &mut Threshold, r: &mut ::idle::Resources) -> (bool, bool) {
    r.USER_UART.claim(t, |user_uart, _t| {
        uart_putstr(user_uart,
            "Entering Level 10 - \"m4loaderhax\"\r\n\r\n\
            You and your friends are partway through the process of pwning a device.\r\n\
            You can control 96 KiB of RAM at 0x20008000, but 8 KiB of encrypted firmware\r\n\
            will load and be executed at 0x20010000. This firmware cannot be changed,\r\n\
            but you have managed to gain the ability to change the decryption key used.\r\n\
            Your friends are convinced that you are stuck. Prove them wrong.\r\n\r\n");
    });

    let mut dec_key = r.PUZZLES_STATE_LORGE.level10_key_orig;

    unsafe { LEVEL10_CLEAR_FLAG = false; }

    // Fill everything else with NOPs
    unsafe {
        for i in 0..((0x20000 - 0x8000) / 4) {
            *((0x20008000 + i * 4) as *mut u32) = 0x46c046c0;   // nop, nop
        }
    }

    // Copy the encrypted code in the middle
    unsafe {
        ::core::ptr::copy_nonoverlapping(
            &r.PUZZLES_STATE_LORGE.level10_code as *const u8,
            0x20010000 as *mut u8, r.PUZZLES_STATE_LORGE.level10_code.len());
    }

    loop {
        r.USER_UART.claim(t, |user_uart, _t| {
            uart_putstr(user_uart,
                "Do you want to:\r\n\
                1) Download the encrypted code (outputs 8192 binary bytes)\r\n\
                2) Upload RAM contents (upload 98304 bytes, but some will be overwritten)\r\n\
                3) Upload new decryption key (16 bytes)\r\n\
                4) Run\r\n\
                Enter choice: ");
        });

        let (rx_c, tx_p, uart, puzzles_state_lorge) = (&mut r.CMDLINE_RX_C, &mut r.CMDLINE_RET_P, &mut r.USER_UART, &r.PUZZLES_STATE_LORGE);

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
                    for i in 0..8192 {
                        uart_putc!(user_uart, puzzles_state_lorge.level10_code[i]);
                    }
                });
            },
            "2" => {
                let load_code_addr = 0x20008000 as *mut u8;
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Upload 98304 bytes now:");

                    // Disable the UART RX interrupt. This disables the line editing
                    // and allows us to directly read characters
                    user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

                    for i in 0..98304 {
                        unsafe {
                            *load_code_addr.offset(i) = uart_getc_manual(user_uart);
                        }
                    }

                    // Need to reenable interrupt
                    user_uart.cr1.modify(|_, w| w.rxneie().bit(true));

                    uart_putstr(user_uart, "\r\n");
                });

                // Copy the encrypted code in the middle
                unsafe {
                    ::core::ptr::copy_nonoverlapping(
                        &r.PUZZLES_STATE_LORGE.level10_code as *const u8,
                        0x20010000 as *mut u8, r.PUZZLES_STATE_LORGE.level10_code.len());
                }
            },
            "3" => {
                uart.claim(t, |user_uart, _t| {
                    uart_putstr(user_uart, "Upload 16 bytes now:");

                    // Disable the UART RX interrupt. This disables the line editing
                    // and allows us to directly read characters
                    user_uart.cr1.modify(|_, w| w.rxneie().bit(false));

                    for i in 0..16 {
                        dec_key[i] = uart_getc_manual(user_uart);
                    }

                    // Need to reenable interrupt
                    user_uart.cr1.modify(|_, w| w.rxneie().bit(true));

                    uart_putstr(user_uart, "\r\n");
                });
            },
            "4" => {
                // Now we decrypt the encrypted code in the middle with AES-CBC
                let aesdec = aes_nostd::aessafe::AesSafe128Decryptor::new(&dec_key);
                for i in (0..(r.PUZZLES_STATE_LORGE.level10_code.len() / 16)).rev() {
                    unsafe {
                        let block = (0x20010000 + 16 * i) as *mut u8;
                        let block_slice = ::core::slice::from_raw_parts_mut(block, 16);
                        let mut dec_out = [0u8; 16];
                        aesdec.decrypt_block(block_slice, &mut dec_out);

                        if i != 0 {
                            let last_block = (0x20010000 + 16 * (i - 1)) as *const u8;
                            let last_block_slice = ::core::slice::from_raw_parts(last_block, 16);

                            for j in 0..16 {
                                dec_out[j] ^= last_block_slice[j];
                            }
                        }

                        block_slice.copy_from_slice(&dec_out)
                    }
                }

                // Jump to the uploaded code
                // Copy a return trampoline to 0x20018000
                unsafe {
                    *(0x20018000 as *mut u32) = 0xdf002000;     // movs r0, #0; svc 0
                }
                // Now actually run the user code
                invoke_user_code(0x20010000, 0x20018001);

                if unsafe { LEVEL10_CLEAR_FLAG } {
                    uart.claim(t, |user_uart, _t| {
                        uart_putstr(user_uart, "SUCCESS!\r\n");
                    });
                    r.PUZZLES_STATE.last_cleared_level = 10;
                    r.PUZZLES_STATE.cleared_levels |= 1 << 10;
                    return (true, true);
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

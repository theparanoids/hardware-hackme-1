#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

# Ensure the device is at the level prompt

import binascii
import serial
import sys
from Crypto.Cipher import AES

if len(sys.argv) < 2:
    print("Usage: {} serport".format(sys.argv[0]))
    sys.exit(1)

con_serport = sys.argv[1]
conser = serial.Serial(con_serport, 115200, timeout=1)

def serwrite(ser, x):
    for b in x:
        ser.write(bytes([b]))
        ser.flush()

WINNING_CODE_BLOB = [0x4f, 0xf6, 0xce, 0x20, 0xcd, 0xf6, 0xad, 0x60,
                     0x46, 0xf2, 0x61, 0x31, 0xc4, 0xf6, 0x20, 0x11,
                     0x46, 0xf6, 0x61, 0x02, 0xc6, 0xf6, 0x20, 0x62,
                     0x46, 0xf6, 0x65, 0x43, 0xc7, 0xf6, 0x20, 0x23,
                     0x46, 0xf6, 0x20, 0x44, 0xc7, 0xf2, 0x65, 0x64,
                     0x42, 0xf2, 0x63, 0x05, 0xc3, 0xf2, 0x30, 0x15,
                     0x46, 0xf2, 0x20, 0x56, 0xc6, 0xf6, 0x64, 0x76,
                     0x46, 0xf2, 0x63, 0x57, 0xc6, 0xf2, 0x78, 0x57,
                     0x00, 0xdf, 0x70, 0x47]

NOP = [0x00, 0xbf]

WINNING_CODE_BLOB = WINNING_CODE_BLOB + NOP * ((96*1024 - len(WINNING_CODE_BLOB)) // 2)
# print(WINNING_CODE_BLOB)

WINNING_CODE_BLOB[0xFFFC - 0x8000] = 0xF8
WINNING_CODE_BLOB[0xFFFD - 0x8000] = 0xF7
WINNING_CODE_BLOB[0xFFFE - 0x8000] = 0x00
WINNING_CODE_BLOB[0xFFFF - 0x8000] = 0xB8

WINNING_CODE_BLOB[0x1FFFC - 0x8000] = 0xE8
WINNING_CODE_BLOB[0x1FFFD - 0x8000] = 0xF7
WINNING_CODE_BLOB[0x1FFFE - 0x8000] = 0x00
WINNING_CODE_BLOB[0x1FFFF - 0x8000] = 0xB8

EXPECTED_MENU = b"Do you want to:\r\n1) Download the encrypted code (outputs 8192 binary bytes)\r\n2) Upload RAM contents (upload 98304 bytes, but some will be overwritten)\r\n3) Upload new decryption key (16 bytes)\r\n4) Run\r\nEnter choice: "

# Get the old code
serwrite(conser, b'1\n')
reply = conser.read(3 + 8192 + len(EXPECTED_MENU))
# print(reply)
assert len(reply) == 3 + 8192 + len(EXPECTED_MENU)
assert reply[:3] == b'1\r\n'
assert reply[3 + 8192:] == EXPECTED_MENU

orig_code = reply[3:3 + 8192]
print(orig_code)

def brutekeykey():
    for b0 in range(0, 0x100):
        for b1 in range(0, 0x100):
            for b2 in range(0, 0x100):
                for b3 in range(0, 0x100):
                    for b4 in range(0, 0x100):
                        for b5 in range(0, 0x100):
                            for b6 in range(0, 0x100):
                                for b7 in range(0, 0x100):
                                    for b8 in range(0, 0x100):
                                        for b9 in range(0, 0x100):
                                            for b10 in range(0, 0x100):
                                                for b11 in range(0, 0x100):
                                                    for b12 in range(0, 0x100):
                                                        for b13 in range(0, 0x100):
                                                            for b14 in range(0, 0x100):
                                                                for b15 in range(0, 0x100):
                                                                    key = bytes([b0, b1, b2, b3, b4, b5, b6, b7,
                                                                                 b8, b9, b10, b11, b12, b13, b14, b15])

                                                                    aes = AES.new(key, AES.MODE_CBC, IV=b'\x00' * 16)
                                                                    msg_dec = aes.decrypt(orig_code)

                                                                    if msg_dec[1] & 0b11111100 == 0b11100100:
                                                                        print(binascii.hexlify(key))
                                                                        print(binascii.hexlify(msg_dec))
                                                                        return key
keykey = brutekeykey()

# Write the new code
serwrite(conser, b'2\n')
expected_reply = b'2\r\nUpload 98304 bytes now:'
reply = conser.read(len(expected_reply))
print(reply)
assert reply == expected_reply
serwrite(conser, WINNING_CODE_BLOB)
reply = conser.read(len(EXPECTED_MENU) + 2)
print(reply)
assert reply == b'\r\n' + EXPECTED_MENU

# Write the new key
serwrite(conser, b'3\n')
expected_reply = b'3\r\nUpload 16 bytes now:'
reply = conser.read(len(expected_reply))
print(reply)
assert reply == expected_reply
serwrite(conser, keykey)

# Go!
reply = conser.read(len(EXPECTED_MENU) + 2)
print(reply)
assert reply == b'\r\n' + EXPECTED_MENU
serwrite(conser, b'4\n')

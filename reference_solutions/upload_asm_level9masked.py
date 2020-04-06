#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import binascii
import serial
import subprocess
import sys

in_serport = sys.argv[1]
in_fn_s = sys.argv[2]

ret = subprocess.call(['arm-none-eabi-gcc', '-Ttext', '0x20008000', '-nostartfiles', in_fn_s])
if ret != 0:
    sys.exit(ret)
ret = subprocess.call(['arm-none-eabi-objcopy', '-O', 'binary', 'a.out'])
if ret != 0:
    sys.exit(ret)
with open('a.out', 'rb') as inf:
    payload = inf.read()
with open('level9maskdump.bin', 'rb') as inf:
    xormask = inf.read()

if len(payload) < 8192:
    payload += b'\x00' * (8192 - len(payload))

payload = bytes([b ^ m for (b, m) in zip(payload, xormask)])

ser = serial.Serial(in_serport, 115200, timeout=1)
print(ser)
print(binascii.hexlify(payload))
# WTF?!
for b in payload:
    ser.write(bytes([b]))
    ser.flush()

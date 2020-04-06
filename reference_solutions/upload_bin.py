#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import binascii
import serial
import subprocess
import sys

in_serport = sys.argv[1]
in_fn = sys.argv[2]

with open(in_fn, 'rb') as f:
	payload = f.read()

ser = serial.Serial(in_serport, 115200, timeout=1)
# WTF?!
for b in payload:
	ser.write(bytes([b]))
	ser.flush()

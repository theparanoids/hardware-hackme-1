#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

# Ensure the device is at the "Enter PIN:" prompt

from serial_port_util import CTFSerial
import sys

if len(sys.argv) < 2:
    print("Usage: {} serport".format(sys.argv[0]))
    sys.exit(1)

in_serport = sys.argv[1]
ser = CTFSerial(in_serport)

for pin in range(10000):
    print("Trying {:04d}".format(pin))
    pinstr = "{:04d}\n".format(pin).encode('ascii')
    ok = ser.write_and_check(pinstr)
    assert ok
    bytes1 = ser.read(1)
    if bytes1 == b'I':
        # "Incorrect PIN!\r\nEnter PIN: "
        reply = ser.read(26)
        assert len(reply) == 26
    else:
        print("That was the correct PIN!")
        break

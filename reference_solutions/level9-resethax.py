#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

# Ensure the device is at the main prompt

import asyncio
import binascii
import os.path
import serial
import sys
import queue
import threading
import time

from gdb_monitor_console import GDBConnection
from serial_port_util import CTFSerial

async def main():
    if len(sys.argv) < 3:
        print("Usage: {} gdb_serport con_serport <start at>".format(sys.argv[0]))
        sys.exit(1)

    gdb_serport = sys.argv[1]
    con_serport = sys.argv[2]

    fut = None

    def gdb_callback(pkt):
        if pkt[0:1] == b'O' and pkt != b'OK':
            print(binascii.unhexlify(pkt[1:]))
        else:
            print(pkt)
            print(fut)

            if fut is not None:
                fut.set_result(pkt)

    gdb = GDBConnection(gdb_serport, gdb_callback)
    con = CTFSerial(con_serport)

    time.sleep(5)

    def get_into_level():
        con.write(b"start 9\n")
        expected_reply = b"start 9\r\nEntering Level 9 - \"secure firmware loading\"\r\n\r\nYou are reverse-engineering a device with encrypted firmware.\r\nThe vendor tells you that the firmware is encrypted with an \"unbreakable\"\r\none-time-pad scheme.\r\n\r\n" + \
                         b"Do you want to:\r\n1) Download the encrypted code (outputs 8192 binary bytes)\r\n2) Upload encrypted code (upload 8192 bytes)\r\nEnter choice: "
        reply = con.read(len(expected_reply))
        # print(expected_reply, reply)
        # assert reply == expected_reply
        if reply != expected_reply:
            print(reply)

    async def do_reset():
        con.ser.reset_input_buffer()

        nonlocal fut
        fut = asyncio.get_running_loop().create_future()
        gdb.send_monitor_command(b'hard_srst')
        await asyncio.wait_for(fut, timeout=1)
        fut = None

        # Make sure we flush out extra data
        reply = con.read(2)

    # Download the original code
    await do_reset()
    get_into_level()

    con.write_and_check(b"1\n")
    orig_code = con.read(8192)
    assert len(orig_code) == 8192
    print(orig_code)

    # Upload the code back
    expected_reply = b"Do you want to:\r\n1) Download the encrypted code (outputs 8192 binary bytes)\r\n2) Upload encrypted code (upload 8192 bytes)\r\nEnter choice: "
    reply = con.read(len(expected_reply))
    if reply != expected_reply:
        print(reply)
    con.write_and_check(b"2\n")
    expected_reply = b"Upload 8192 bytes now:"
    reply = con.read(len(expected_reply))
    if reply != expected_reply:
        print(reply)
    con.write(orig_code)

    # Now dump data from RAM
    fut = asyncio.get_running_loop().create_future()
    gdb.send_monitor_command(b'connect_srst enable')
    await asyncio.wait_for(fut, timeout=1)

    fut = asyncio.get_running_loop().create_future()
    gdb.send_monitor_command(b'swdp_scan')
    await asyncio.wait_for(fut, timeout=1)

    fut = asyncio.get_running_loop().create_future()
    gdb.send_packet(b'vAttach;1')
    await asyncio.wait_for(fut, timeout=1)

    # XXX why is this broken?
    fut = asyncio.get_running_loop().create_future()
    gdb.send_packet(b'm20000000,1')
    await asyncio.wait_for(fut, timeout=1)

    ramdump = b''
    for i in range(0x2000 // 0x200):
        fut = asyncio.get_running_loop().create_future()
        readcmd = 'm{:X},200'.format(0x20008000 + i * 0x200).encode('ascii')
        print(readcmd)
        gdb.send_packet(readcmd)
        await asyncio.wait_for(fut, timeout=1)
        ramblock = fut.result()
        assert len(ramblock) == 0x200 * 2
        ramdump += ramblock

    ramdump = binascii.unhexlify(ramdump)
    assert len(ramdump) == 8192
    print(ramdump)

    rammask = bytes((x ^ y for x, y in zip(orig_code, ramdump)))
    print(rammask)
    with open('level9maskdump.bin', 'wb') as f:
        f.write(rammask)

    fut = None
    gdb.shutdown()
    con.ser.close()

    print("***** UNPLUG *****")
    while os.path.exists(con_serport) or os.path.exists(gdb_serport):
        pass

    print("***** REPLUG *****")
    while not os.path.exists(con_serport) or not os.path.exists(gdb_serport):
        pass

    con = CTFSerial(con_serport)
    get_into_level()

    WINNING_DUMP_CODE = [0x4b, 0xf6, 0xef, 0x60, 0xcd, 0xf6, 0xad, 0x60,
                         0x46, 0xf6, 0x6f, 0x41, 0xc5, 0xf2, 0x6e, 0x51,
                         0x42, 0xf2, 0x4c, 0x02, 0xc6, 0xf2, 0x6b, 0x32,
                         0x46, 0xf2, 0x6c, 0x53, 0xc6, 0xf2, 0x76, 0x53,
                         0x42, 0xf2, 0x70, 0x04, 0xc2, 0xf2, 0x39, 0x04,
                         0x47, 0xf6, 0x3f, 0x05, 0xc6, 0xf6, 0x30, 0x45,
                         0x47, 0xf2, 0x68, 0x46, 0xc2, 0xf2, 0x6b, 0x06,
                         0x46, 0xf2, 0x69, 0x17, 0xc7, 0xf6, 0x62, 0x07,
                         0x00, 0xdf, 0x00, 0x20, 0x00, 0xdf]

    WINNING_DUMP_CODE = WINNING_DUMP_CODE + [0] * (8192 - len(WINNING_DUMP_CODE))

    coded_bytes = bytes((x ^ y for x, y in zip(WINNING_DUMP_CODE, rammask)))
    print(coded_bytes)

    con.write_and_check(b"2\n")
    expected_reply = b"Upload 8192 bytes now:"
    reply = con.read(len(expected_reply))
    if reply != expected_reply:
        print(reply)

    con.write(coded_bytes)

if __name__=='__main__':
    asyncio.run(main())

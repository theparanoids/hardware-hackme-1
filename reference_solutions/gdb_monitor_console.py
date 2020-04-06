#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import binascii
import queue
import serial
import sys
import threading


def rxthread(ser, quitevt, pktcb, acksem):
    """Handle receiving data according to GDB packet framing

    ser: opened serial.Serial object for serial port (timeout must be finite)
    quitevt: threading.Event object telling thread to quit
    pktcb: Function that will be called when a complete packet is received
    acksem: threading.Semaphore that will be released when an ack is received
    """
    IDLE = 0
    IN_PACKET = 1
    CSUM0 = 2
    CSUM1 = 3

    state = IDLE

    while not quitevt.is_set():
        c = ser.read(1)
        if len(c) == 0:
            continue

        if state == IDLE:
            if c == b'+':
                # Ack for our command
                acksem.release()
            elif c == b'$':
                cur_packet = b''
                state = IN_PACKET
        elif state == IN_PACKET:
            if c == b'#':
                state = CSUM0
            else:
                cur_packet += c
        elif state == CSUM0:
            csum = int(c, 16) << 4
            state = CSUM1
        elif state == CSUM1:
            csum |= int(c, 16)
            actual_csum = compute_packet_csum(cur_packet)
            if csum == actual_csum:
                # ACK
                ser.write(b'+')
                pktcb(cur_packet)
            else:
                # NAK
                ser.write(b'-')
            state = IDLE
        else:
            assert False


def compute_packet_csum(pkt):
    """Computes the checksum for the given GDB packet"""
    csum = 0
    for x in pkt:
        csum += x
    csum = csum & 0xFF
    return csum


def wrap_packet(pkt):
    """Add checksum to the given packet"""
    csum = compute_packet_csum(pkt)
    return b"$" + pkt + b"#" + binascii.hexlify(bytes([csum]))


def wrap_monitor_command(cmd):
    """Wraps the given "monitor" command with GDB packet framing"""
    cmd_hex = binascii.hexlify(cmd)
    cmd_wrapped = b"qRcmd," + cmd_hex
    return wrap_packet(cmd_wrapped)


class GDBConnection:
    def __init__(self, device_node_name, rx_callback):
        """Open a connection to a GDB stub and start background thread"""
        self.ser = serial.Serial(device_node_name, 115200, timeout=1)

        self.quitevt = threading.Event()
        self.acksem = threading.Semaphore(0)

        self.rxthreadobj = threading.Thread(
            target=rxthread,
            args=(self.ser, self.quitevt, rx_callback, self.acksem))
        self.rxthreadobj.start()

    def shutdown(self):
        """Shut down the background receive thread"""
        self.quitevt.set()
        self.rxthreadobj.join()
        self.ser.close()

    def send_packet(self, pkt):
        """Send a raw packet to the remote stub"""
        cmd_pkt = wrap_packet(pkt)

        for x in cmd_pkt:
            self.ser.write(bytes([x]))
            self.ser.flush()

        self.acksem.acquire()

    def send_monitor_command(self, cmd):
        """Send a "monitor" command to the remote stub"""
        cmd_pkt = wrap_monitor_command(cmd)

        for x in cmd_pkt:
            self.ser.write(bytes([x]))
            self.ser.flush()

        self.acksem.acquire()


def main():
    """Demo function that allows interactively entering "monitor" commands
    and viewing the data that is printed by the debug stub"""

    def handle_rx_packet(pkt):
        """Ignore ACK packets and print the contents of "O" packets"""
        if pkt == b'OK':
            pass
        elif pkt[:1] == b'O':
            pkt = pkt[1:]
            sys.stdout.buffer.write(binascii.unhexlify(pkt))
            sys.stdout.buffer.flush()
        else:
            print("Unknown '{}'".format(pkt))

    if len(sys.argv) < 2:
        print("Usage: {} /dev/<serialport>".format(sys.argv[0]))
        return

    serport = sys.argv[1]
    gdb = GDBConnection(serport, handle_rx_packet)

    try:
        while True:
            line = input().encode('utf-8')
            gdb.send_monitor_command(line)
    except KeyboardInterrupt:
        gdb.shutdown()


if __name__ == '__main__':
    main()

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import serial


class CTFSerial:
    def __init__(self, device_node_name):
        """Open a connection to the hackme target"""
        self.ser = serial.Serial(device_node_name, 115200, timeout=1)

    def write(self, b):
        """Write bytes to the target.

        This automatically issues a flush after writing each byte as a
        workaround for unknown flakiness in the TTY layer"""
        for byte in b:
            self.ser.write(bytes([byte]))
            self.ser.flush()

    def read(self, n):
        """Read bytes from the target. Directly returns the data read."""
        return self.ser.read(n)

    def write_and_check(self, b):
        """Writes bytes to the target. Waits for the board to echo back the
        data that was read. Checks to ensure that the echo-ed data is as
        expected"""
        bytes_with_all_crlf = CTFSerial._crlf_hack(b)

        self.write(b)
        read = self.read(len(bytes_with_all_crlf))
        return read == bytes_with_all_crlf

    def _crlf_hack(inp):
        r"""Helper to convert all forms of line endings into CRLF in the same
        way as the hackme itself will

        >>> CTFSerial._crlf_hack(b'test\n')
        b'test\r\n'
        >>> CTFSerial._crlf_hack(b'test\r')
        b'test\r\n'
        >>> CTFSerial._crlf_hack(b'test\r\n')
        b'test\r\n'
        >>> CTFSerial._crlf_hack(b'test\n\n')
        b'test\r\n\r\n'
        >>> CTFSerial._crlf_hack(b'test\r\r')
        b'test\r\n\r\n'
        >>> CTFSerial._crlf_hack(b'test\n\r')
        b'test\r\n\r\n'
        >>> CTFSerial._crlf_hack(b'test\nfoobar')
        b'test\r\nfoobar'
        >>> CTFSerial._crlf_hack(b'test\rfoobar')
        b'test\r\nfoobar'
        >>> CTFSerial._crlf_hack(b'test\r\nfoobar')
        b'test\r\nfoobar'
        >>> CTFSerial._crlf_hack(b'test\n\nfoobar')
        b'test\r\n\r\nfoobar'
        >>> CTFSerial._crlf_hack(b'test\r\rfoobar')
        b'test\r\n\r\nfoobar'
        >>> CTFSerial._crlf_hack(b'test\n\rfoobar')
        b'test\r\n\r\nfoobar'
        """

        outp = b''
        seencr = False

        for b in inp:
            if b != ord('\r') and b != ord('\n'):
                if seencr:
                    outp += b'\r\n'
                seencr = False
                outp += bytes([b])
            elif b == ord('\r'):
                if seencr:
                    outp += b'\r\n'
                seencr = True
            elif b == ord('\n'):
                outp += b'\r\n'
                seencr = False

        if seencr:
            outp += b'\r\n'

        return outp

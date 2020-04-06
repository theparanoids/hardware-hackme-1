# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import serial
import sys

if len(sys.argv) < 2:
    print("Usage: {} con_serport".format(sys.argv[0]))
    sys.exit(1)

con_serport = sys.argv[1]
conser = serial.Serial(con_serport, 115200, timeout=1)

ERASE = b'__unlockme'
REPLY = b'__unlockme'
conser.write(ERASE)
print(conser.read(len(REPLY)))

ERASE = b'_this'
REPLY = b'_this'
conser.write(ERASE)
print(conser.read(len(REPLY)))

ERASE = b'_will'
REPLY = b'_will'
conser.write(ERASE)
print(conser.read(len(REPLY)))

ERASE = b'_brick'
REPLY = b'_brick'
conser.write(ERASE)
print(conser.read(len(REPLY)))

ERASE = b'_the'
REPLY = b'_the'
conser.write(ERASE)
print(conser.read(len(REPLY)))

ERASE = b'_device\n'
REPLY = b'_device\r\n'
conser.write(ERASE)
print(conser.read(len(REPLY)))


print(con_serport)

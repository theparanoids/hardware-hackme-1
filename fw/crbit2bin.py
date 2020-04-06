#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import sys

infn = sys.argv[1]
outfn = sys.argv[2]

with open(infn, 'rb') as f:
	crbitlines = f.readlines()

crbitlines = [x.strip() for x in crbitlines if x.strip()]
crbitlines = [x for x in crbitlines if not x.startswith(b'//')]

# print(crbitlines)
assert(len(crbitlines) == 50)

outbytes = b''

for addr in range(49):
	line = crbitlines[addr]
	assert len(line) == 260

	# print(line)
	# line = line[::-1]

	bytesout = [0] * 33

	for biti in range(260):
		if line[biti] == ord('1'):
			bytesout[biti // 8] |= 1 << (biti % 8)

	# print(bytesout)
	outbytes += bytes(bytesout)

# print(outbytes)
with open(outfn, 'wb') as f:
	f.write(outbytes)

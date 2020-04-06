#!/usr/bin/env python3

# Copyright 2020, Verizon Media
# Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

import base64
import os
import subprocess
import tempfile
import pyasn1.codec.ber.decoder as asn1decoder
import collections
import hashlib
import secrets
import serial
import sys
import json
import time
from Crypto.Cipher import AES

LEVEL9_CODE = r"""
.syntax unified
.code 16

// R7 = USART2 (user)
movw r0, #0x4400
movt r0, #0x4000

adr r1, hello
bl putstr

mainloop:
    movs r1, #'/'
    bl putchar
    movs r1, #8
    bl putchar
    movs r1, #'-'
    bl putchar
    movs r1, #8
    bl putchar
    movs r1, #'\\'
    bl putchar
    movs r1, #8
    bl putchar
    movs r1, #'|'
    bl putchar
    movs r1, #8
    bl putchar
    b mainloop

# r0 = uart
# r1 = char
putchar:
    ldr r2, [r0]
    tst r2, 0x80
    beq putchar
    strb r1, [r0, 4]
    bx lr

# r0 = uart
getchar:
    ldr r1, [r0]
    tst r1, 0x20
    beq getchar
    ldrb r0, [r0, 4]
    bx lr

# r0 = uart
# r1 = string
putstr:
    push {r4, lr}
    movs r4, r1
loop:
    ldrb r1, [r4]
    orrs r1, r1
    it eq
    popeq {r4, pc}
    bl putchar
    adds r4, #1
    b loop

hello: .asciz "Hello World\r\nThis is the Level 9 default code.\r\nYou need to hack me!\r\n\r\n"
"""

LEVEL10_CODE = r"""
.syntax unified
.code 16

// R7 = USART2 (user)
movw r0, #0x4400
movt r0, #0x4000

adr r1, hello
bl putstr

mainloop:
    movs r1, #'/'
    bl putchar
    movs r1, #8
    bl putchar
    movs r1, #'|'
    bl putchar
    movs r1, #8
    bl putchar
    movs r1, #'\\'
    bl putchar
    movs r1, #8
    bl putchar
    movs r1, #'-'
    bl putchar
    movs r1, #8
    bl putchar
    b mainloop

# r0 = uart
# r1 = char
putchar:
    ldr r2, [r0]
    tst r2, 0x80
    beq putchar
    strb r1, [r0, 4]
    bx lr

# r0 = uart
getchar:
    ldr r1, [r0]
    tst r1, 0x20
    beq getchar
    ldrb r0, [r0, 4]
    bx lr

# r0 = uart
# r1 = string
putstr:
    push {r4, lr}
    movs r4, r1
loop:
    ldrb r1, [r4]
    orrs r1, r1
    it eq
    popeq {r4, pc}
    bl putchar
    adds r4, #1
    b loop

hello: .asciz "Hello World\r\nThis is the Level 10 default code.\r\nYou need to hack me!\r\n\r\n"
"""

def my_random(n):
    ret = 0
    while ret == 0:
        ret = secrets.randbelow(n)
    return ret

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


# Keypair generation and ECDSA ################################################

def hash_message(message):
    """Returns the truncated SHA256 hash of the message."""
    message_hash = hashlib.sha256(message).digest()
    e = int.from_bytes(message_hash, 'big')

    # FIPS 180 says that when a hash needs to be truncated, the rightmost bits
    # should be discarded.
    if e.bit_length() > curve.n.bit_length():
        z = e >> (e.bit_length() - curve.n.bit_length())
    else:
        z = e

    assert z.bit_length() <= curve.n.bit_length()

    return z


def sign_message_insecurely(private_key, message, k):
    z = hash_message(message)

    r = 0
    s = 0

    while not r or not s:
        x, y = scalar_mult(k, curve.g)

        r = x % curve.n
        s = ((z + r * private_key) * inverse_mod(k, curve.n)) % curve.n

    return (r, s)


def setup_rsa(tmpdir, everything, CHIPID):
    # Run openssl to generate key
    subprocess.run(['openssl', 'genrsa', '-out', tmpdir + '/' + 'rsapriv.pem', '2048'], check=True)

    with open(tmpdir + '/' + 'rsapriv.pem', 'r') as f:
        rsapriv_pem = f.read()
    # print(rsapriv_pem)
    everything['rsa_privk'] = rsapriv_pem
    rsapriv_der = base64.b64decode(''.join(rsapriv_pem.splitlines()[1:-1]))
    # print(rsapriv_der)
    rsapriv_asn1 = asn1decoder.decode(rsapriv_der)
    assert rsapriv_asn1[1] == b''
    rsapriv_asn1 = rsapriv_asn1[0]

    n = int(rsapriv_asn1[1])
    # print(n)
    r = (2**2048) % n
    rr = (r**2) % n
    # print(r)
    # print(rr)
    everything['rsa_n'] = n
    everything['rsa_r'] = r
    everything['rsa_rr'] = rr

    lvl7_file = "# ACME Device Provisioning System\nDEVICE_ID={}\nLOCKED_STATE=true".format(CHIPID).encode('ascii')
    lvl7_file = lvl7_file + b" " * (128 - len(lvl7_file))
    # print(lvl7_file)

    # Sign file
    with open(tmpdir + '/' + 'rsapayload.txt', 'wb') as f:
        f.write(lvl7_file)
    subprocess.run(['openssl', 'dgst', '-sha256', '-sign', tmpdir + '/' + 'rsapriv.pem',
        '-out', tmpdir + '/' + 'rsapayloadsig.bin', tmpdir + '/' + 'rsapayload.txt'], check=True)

    with open(tmpdir + '/' + 'rsapayloadsig.bin', 'rb') as f:
        lvl7_sig = f.read()

    everything['level7_example'] = base64.b64encode(lvl7_sig + lvl7_file).decode('ascii')


def setup_ecdsa(everything, CHIPID):
    ecdsa_mont_R = (2**256) % curve.p
    ecdsa_privk = my_random(curve.n)
    ecdsa_randk = my_random(curve.n)
    everything['ecdsa_privk'] = ecdsa_privk
    everything['ecdsa_randk'] = ecdsa_randk
    ecdsa_pubk = scalar_mult(ecdsa_privk, curve.g)
    print("Public key (Not Montgomery): x = {:64x}, y = {:64x}".format(*ecdsa_pubk))
    ecdsa_pubk_plus_g = point_add(ecdsa_pubk, curve.g)
    print("Public key + G (Not Montgomery): x = {:64x}, y = {:64x}".format(*ecdsa_pubk_plus_g))

    ecdsa_pubk_mont = ((ecdsa_pubk[0] * ecdsa_mont_R) % curve.p, (ecdsa_pubk[1] * ecdsa_mont_R) % curve.p)
    ecdsa_pubk_plus_g_mont = ((ecdsa_pubk_plus_g[0] * ecdsa_mont_R) % curve.p, (ecdsa_pubk_plus_g[1] * ecdsa_mont_R) % curve.p)
    print("Public key (Montgomery): x = {:64x}, y = {:64x}".format(*ecdsa_pubk_mont))
    print("Public key + G (Montgomery): x = {:64x}, y = {:64x}".format(*ecdsa_pubk_plus_g_mont))
    everything['level8_pubk_x'] = ecdsa_pubk_mont[0]
    everything['level8_pubk_y'] = ecdsa_pubk_mont[1]
    everything['level8_pubk_plus_g_x'] = ecdsa_pubk_plus_g_mont[0]
    everything['level8_pubk_plus_g_y'] = ecdsa_pubk_plus_g_mont[1]

    lvl8_file1 = "# RR Device Provisioning System\nDEVICE_ID={}\nLOCKED_STATE=true".format(CHIPID).encode('ascii')
    lvl8_file1 = lvl8_file1 + b" " * (128 - len(lvl8_file1))
    lvl8_file2 = "# RR Device Provisioning System\nDEVICE_ID={}\nLOCKED_STATE=true".format("81C17E1F29E67251E6461147").encode('ascii')
    lvl8_file2 = lvl8_file2 + b" " * (128 - len(lvl8_file2))

    lvl8_sig1 = sign_message_insecurely(ecdsa_privk, lvl8_file1, ecdsa_randk)
    lvl8_sig2 = sign_message_insecurely(ecdsa_privk, lvl8_file2, ecdsa_randk)

    print("sig 1 r = {:64x}, s = {:64x}".format(*lvl8_sig1))
    print("sig 2 r = {:64x}, s = {:64x}".format(*lvl8_sig2))

    lvl8_file1 = lvl8_sig1[0].to_bytes(32, byteorder='big') + lvl8_sig1[1].to_bytes(32, byteorder='big') + lvl8_file1
    lvl8_file2 = lvl8_sig2[0].to_bytes(32, byteorder='big') + lvl8_sig2[1].to_bytes(32, byteorder='big') + lvl8_file2

    everything['level8_file1'] = base64.b64encode(lvl8_file1).decode('ascii')
    everything['level8_file2'] = base64.b64encode(lvl8_file2).decode('ascii')


def setup_xorcrypto(tmpdir, everything):
    with open(tmpdir + '/' + 'level9.S', 'w') as f:
        f.write(LEVEL9_CODE)
    subprocess.run(['arm-none-eabi-gcc', '-Ttext', '0x20008000', '-nostartfiles',
        '-o', tmpdir + '/' + 'level9.bin', tmpdir + '/' + 'level9.S'], check=True)
    subprocess.run(['arm-none-eabi-objcopy', '-O', 'binary',
        tmpdir + '/' + 'level9.bin'], check=True)
    with open(tmpdir + '/' + 'level9.bin', 'rb') as f:
        level9_binary = f.read()
    #print(level9_binary)

    level9_binary = level9_binary + secrets.token_bytes(8192 - len(level9_binary))
    level9_mask = secrets.token_bytes(8192)
    level9_binary_enc = bytes([x ^ y for (x, y) in zip(level9_binary, level9_mask)])

    everything['level9_binary'] = base64.b64encode(level9_binary_enc).decode('ascii')
    everything['level9_mask'] = base64.b64encode(level9_mask).decode('ascii')


def setup_aescrypto(tmpdir, everything):
    with open(tmpdir + '/' + 'level10.S', 'w') as f:
        f.write(LEVEL10_CODE)
    subprocess.run(['arm-none-eabi-gcc', '-Ttext', '0x20008000', '-nostartfiles',
        '-o', tmpdir + '/' + 'level10.bin', tmpdir + '/' + 'level10.S'], check=True)
    subprocess.run(['arm-none-eabi-objcopy', '-O', 'binary',
        tmpdir + '/' + 'level10.bin'], check=True)
    with open(tmpdir + '/' + 'level10.bin', 'rb') as f:
        level10_binary = f.read()
    #print(level10_binary)

    level10_binary = level10_binary + secrets.token_bytes(8192 - len(level10_binary))
    level10_key = secrets.token_bytes(16)
    aes = AES.new(level10_key, AES.MODE_CBC, IV=b'\x00' * 16)
    level10_binary_enc = aes.encrypt(level10_binary)

    everything['level10_binary'] = base64.b64encode(level10_binary_enc).decode('ascii')
    everything['level10_key'] = base64.b64encode(level10_key).decode('ascii')

def main():
    if len(sys.argv) < 2:
        print("Usage: {} con_serport".format(sys.argv[0]))
        sys.exit(1)

    everything = {}

    con_serport = sys.argv[1]
    # outdata_fn = sys.argv[2]
    conser = serial.Serial(con_serport, 115200, timeout=1)

    conser.write(b'__provision\n')
    expected_reply = b'__provision\r\nDEVICE_ID: XXXXXXXXXXXXXXXXXXXXXXXX\r\n'
    reply = conser.read(len(expected_reply))
    assert reply[:24] == expected_reply[:24]
    assert reply[-2:] == b'\r\n'
    chipid = reply[24:48]
    print(chipid)

    CHIPID = chipid.decode('ascii')
    everything['CHIPID'] = CHIPID

    with tempfile.TemporaryDirectory() as tmpdir:
        print(tmpdir)

        ##### For level 4 (RNG) #####
        everything['level4_rng_seed'] = base64.b64encode(secrets.token_bytes(40)).decode('ascii')

        ##### For level 7 (RSA) #####
        setup_rsa(tmpdir, everything, CHIPID)

        ##### For level 8 (ECDSA) #####
        setup_ecdsa(everything, CHIPID)

        ##### For level 9 (XOR crypto) #####
        setup_xorcrypto(tmpdir, everything)

        ##### For level 10 (AES crypto) #####
        setup_aescrypto(tmpdir, everything)

    outdata_fn = 'provision-record-{}.json'.format(CHIPID)

    # print(everything)
    with open(outdata_fn, 'w') as f:
        json.dump(everything, f)

    print("Sending RNG seed")
    xx = base64.b64decode(everything['level4_rng_seed'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending RSA N")
    xx = everything['rsa_n'].to_bytes(256, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending RSA R")
    xx = everything['rsa_r'].to_bytes(256, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending RSA R^2")
    xx = everything['rsa_rr'].to_bytes(256, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending RSA example")
    xx = base64.b64decode(everything['level7_example'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending ECDSA pubk x")
    xx = everything['level8_pubk_x'].to_bytes(32, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    print("Sending ECDSA pubk y")
    xx = everything['level8_pubk_y'].to_bytes(32, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending ECDSA pubk + G x")
    xx = everything['level8_pubk_plus_g_x'].to_bytes(32, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    print("Sending ECDSA pubk + G y")
    xx = everything['level8_pubk_plus_g_y'].to_bytes(32, byteorder='big')
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending ECDSA example 1")
    xx = base64.b64decode(everything['level8_file1'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    print("Sending ECDSA example 2")
    xx = base64.b64decode(everything['level8_file2'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending Level 9 code")
    xx = base64.b64decode(everything['level9_binary'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    print("Sending Level 9 XOR")
    xx = base64.b64decode(everything['level9_mask'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending Level 10 code")
    xx = base64.b64decode(everything['level10_binary'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    print("Sending Level 10 key")
    xx = base64.b64decode(everything['level10_key'])
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending hardcoded answers")
    xx = b'\x02\x08\x09\x01'
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    xx = b'\x22\x68\x1f\x5d\xd3\x19\x6c\x1f'
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    xx = b'\x14\x1e\xfa\xea\x79\xfc\x06\xd1'
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'
    xx = b'The magic words weren\'t squeamish ossifrage this time. Disappointing..'
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    print("Sending CTF flags")
    xx = b'06fcd545ac08354819580ae94aa92756' + \
         b'1a36cdf3f815ddafa1ceaa9319f5c84c' + \
         b'211b5ddd566037617bdd65c3f1d4836c' + \
         b'4f07a3cf9623c47ec45bd5c49e572f48' + \
         b'c7c4d371a8f6630da41e986953dc55aa' + \
         b'81fa61160e7cfa362fc49bb327253aa5' + \
         b'1ab95018bfa3b3c7582d31cb72d95841' + \
         b'b53bae63a361fcf84738cb92adca5985' + \
         b'54f2a75097d2f5d61b3cb96529adf614' + \
         b'cc4257be79040bb8258f8f01f6626766' + \
         b'2b8c3ee77b9c51cfc1c88fb41b182fde' + \
         b'2dee747c42138b13b46a8d5a1ddfb34a' + \
         b'fd2b89e81036bdb9d88cdcb0db278fec' + \
         b'7217096b9d12fb45393a878261b02ce0' + \
         b'9e94493be2c112825e9a8ac6a29299ee' + \
         b'0add0dc74ccb92205f770bf83f7cc5ea'
    for x in xx:
        conser.write(bytes([x]))
        conser.flush()
    reply = conser.read(1)
    assert reply == b'O'

    time.sleep(3)
    reply = conser.read(2)
    print(reply)
    assert reply == b'> '

    if not (len(sys.argv) >= 3 and sys.argv[2] == "nolock"):
        conser.write(b'__lockme\r\n')
        expected_reply = b'__lockme\r\n'
        reply = conser.read(len(expected_reply))
        print(reply)
        assert reply == expected_reply

if __name__=='__main__':
    main()

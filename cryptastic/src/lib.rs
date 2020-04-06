// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

#![no_std]

#[cfg(test)]
extern crate std;

// Multiplicative inverse of numbers mod 256. Note that only odd numbers have
// a multiplicative inverse because only odd numbers are coprime to 256.
// This table can be computed by using the Extended Euclidean algorithm, but
// in practice this table was computed using brute-force:
// invlist = []
// for i in range(1, 256, 2):
//     inv = None
//     for invguess in range(256):
//         if (i * invguess) & 0xFF == 1:
//             inv = invguess
//             break
//     assert inv is not None
//     invlist.append(inv)
pub const MODINV_BYTES: [u8; 128] = [
    0x01, 0xAB, 0xCD, 0xB7, 0x39, 0xA3, 0xC5, 0xEF,
    0xF1, 0x1B, 0x3D, 0xA7, 0x29, 0x13, 0x35, 0xDF,
    0xE1, 0x8B, 0xAD, 0x97, 0x19, 0x83, 0xA5, 0xCF,
    0xD1, 0xFB, 0x1D, 0x87, 0x09, 0xF3, 0x15, 0xBF,
    0xC1, 0x6B, 0x8D, 0x77, 0xF9, 0x63, 0x85, 0xAF,
    0xB1, 0xDB, 0xFD, 0x67, 0xE9, 0xD3, 0xF5, 0x9F,
    0xA1, 0x4B, 0x6D, 0x57, 0xD9, 0x43, 0x65, 0x8F,
    0x91, 0xBB, 0xDD, 0x47, 0xC9, 0xB3, 0xD5, 0x7F,
    0x81, 0x2B, 0x4D, 0x37, 0xB9, 0x23, 0x45, 0x6F,
    0x71, 0x9B, 0xBD, 0x27, 0xA9, 0x93, 0xB5, 0x5F,
    0x61, 0x0B, 0x2D, 0x17, 0x99, 0x03, 0x25, 0x4F,
    0x51, 0x7B, 0x9D, 0x07, 0x89, 0x73, 0x95, 0x3F,
    0x41, 0xEB, 0x0D, 0xF7, 0x79, 0xE3, 0x05, 0x2F,
    0x31, 0x5B, 0x7D, 0xE7, 0x69, 0x53, 0x75, 0x1F,
    0x21, 0xCB, 0xED, 0xD7, 0x59, 0xC3, 0xE5, 0x0F,
    0x11, 0x3B, 0x5D, 0xC7, 0x49, 0x33, 0x55, 0xFF
];

// Compute the modular multiplicative inverse of a mod 2^32. This computation
// relies on the fact that ax ≡ 1 mod 2^k ==> ax(2-ax) ≡ 1 mod 2^{2k}
// This trick was learned from https://crypto.stackexchange.com/a/47496
// which references a post from sci.crypt
pub fn modinv32(a: u32) -> u32 {
    let a8: u8 = a as u8;
    let a16: u16 = a as u16;
    debug_assert!(a8 % 2 != 0);
    // First, the modular multiplicative inverse of a mod 2^8 is found using a
    // look-up table
    let x8: u8 = MODINV_BYTES[(a8 / 2) as usize];
    // The above trick is applied once to find the modular multiplicative
    // inverse of a mod 2^16
    let x16: u16 = (x8 as u16).wrapping_mul(2u16.wrapping_sub((x8 as u16).wrapping_mul(a16)));
    // Apply the trick again to find the modular multiplicative inverse of
    // a mod 2^32 as desired
    let x32: u32 = (x16 as u32).wrapping_mul(2u32.wrapping_sub((x16 as u32).wrapping_mul(a)));

    x32
}

pub struct Bignum2048(pub [u32; 64]);
pub struct Bignum2048Oversized(pub [u32; 65]);
pub struct Bignum4096Oversized(pub [u32; 129]);

// Add b to a, modifying a and returning carry
// Adds up to length of a only. If b is larger, extra is ignored.
// If b is smaller, it is zero-extended.
fn addhelper(a: &mut [u32], b: &[u32]) -> bool {
    debug_assert!(a.len() > 0);
    debug_assert!(b.len() > 0);

    let (result, mut carryout) = a[0].overflowing_add(b[0]);
    a[0] = result;

    for i in 1..a.len() {
        let carryin = carryout;

        let b = if i < b.len() { b[i] } else { 0 };

        let (mut result_intermed, mut carry_intermed) = a[i].overflowing_add(b);
        if carryin {
            let (result_carry, carry_carry) = result_intermed.overflowing_add(1);
            result_intermed = result_carry;
            carry_intermed = carry_carry || carry_intermed;
        }
        a[i] = result_intermed;
        carryout = carry_intermed;
    }

    carryout
}

// Subtract b from a, modifying a and returning borrow (a will become a - b)
// Subtracts up to length of a only. If b is larger, extra is ignored.
// If b is smaller, it is zero-extended (not sign-extended!).
fn subhelper(a: &mut [u32], b: &[u32]) -> bool {
    debug_assert!(a.len() > 0);
    debug_assert!(b.len() > 0);

    let (result, mut borrowout) = a[0].overflowing_sub(b[0]);
    a[0] = result;

    for i in 1..a.len() {
        let borrowin = borrowout;

        let b = if i < b.len() { b[i] } else { 0 };

        let (mut result_intermed, mut borrow_intermed) = a[i].overflowing_sub(b);
        if borrowin {
            let (result_borrow, borrow_borrow) = result_intermed.overflowing_sub(1);
            result_intermed = result_borrow;
            borrow_intermed = borrow_borrow || borrow_intermed;
        }
        a[i] = result_intermed;
        borrowout = borrow_intermed;
    }

    borrowout
}

// Returns if a is less than b, equal to b, or greater than b
// So ::core::cmp::Ordering::Less means a < b
// Zero-extends
fn cmphelper(a: &[u32], b: &[u32]) -> ::core::cmp::Ordering {
    debug_assert!(a.len() > 0);
    debug_assert!(b.len() > 0);

    let max_len = ::core::cmp::max(a.len(), b.len());

    for i in (0..max_len).rev() {
        let a = if i < a.len() { a[i] } else { 0 };
        let b = if i < b.len() { b[i] } else { 0 };

        if a < b {
            return ::core::cmp::Ordering::Less;
        }

        if a > b {
            return ::core::cmp::Ordering::Greater;
        }
    }

    return ::core::cmp::Ordering::Equal;
}

// Multiplies a * b and stores in out. out must be _exactly_ the correct size (not over).
fn mulhelper(out: &mut [u32], a: &[u32], b: &[u32]) {
    debug_assert!(out.len() == a.len() + b.len());

    for i in 0..out.len() {
        out[i] = 0;
    }

    for b_i in 0..b.len() {
        for a_i in 0..a.len() {
            let temp64: u64 = (a[a_i] as u64) * (b[b_i] as u64);
            let temp32l: u32 = temp64 as u32;
            let temp32h: u32 = (temp64 >> 32) as u32;

            let idx: usize = a_i + b_i;
            let carry = addhelper(&mut out[idx..], &[temp32l, temp32h]);
            debug_assert!(!carry);
        }
    }
}

// Shift the input right by 1 bit (dividing by 2). Sign-extends!
fn shr1helper(x: &mut [u32]) {
    let mut shift_in_bit = x[x.len() - 1] & 0x80000000 != 0;

    for i in (0..x.len()).rev() {
        let shift_out_bit = x[i] & 1 != 0;
        x[i] = (x[i] >> 1) | if shift_in_bit { 0x80000000 } else {0};
        shift_in_bit = shift_out_bit;
    }
}

// Check whether a bignum is equal to 0
fn iszero(x: &[u32]) -> bool {
    for &xi in x {
        if xi != 0 {
            return false;
        }
    }

    return true;
}

// Return a * b mod n using Montgomery multiplication
pub fn mont_mul(a: &Bignum2048, b: &Bignum2048, n: &Bignum2048,
    work_bignum_4096_oversized: &mut Bignum4096Oversized,
    work_bignum_2048_oversized: &mut Bignum2048Oversized) -> Bignum2048 {

    let x = work_bignum_4096_oversized;
    x.0[128] = 0;
    mulhelper(&mut x.0[..128], &a.0, &b.0);
    let work_kn = work_bignum_2048_oversized;
    let result = mont_core(&mut x.0, &n.0, &mut work_kn.0);
    let mut ret: Bignum2048 = Bignum2048([0; 64]);
    ret.0.copy_from_slice(result);
    ret
}

#[allow(non_upper_case_globals)]
pub mod secp256r1 {
    // Field prime
    pub const P: [u32; 8] = [0xffffffff, 0xffffffff, 0xffffffff, 0x00000000, 0x00000000, 0x00000000, 0x00000001, 0xffffffff];
    // Curve equation
    // pub const A: [u32; 8] = [0xfffffffc, 0xffffffff, 0xffffffff, 0x00000000, 0x00000000, 0x00000000, 0x00000001, 0xffffffff];
    // pub const B: [u32; 8] = [0x27d2604b, 0x3bce3c3e, 0xcc53b0f6, 0x651d06b0, 0x769886bc, 0xb3ebbd55, 0xaa3a93e7, 0x5ac635d8];
    // Subgroup order
    pub const N: [u32; 8] = [0xfc632551, 0xf3b9cac2, 0xa7179e84, 0xbce6faad, 0xffffffff, 0xffffffff, 0x00000000, 0xffffffff];

    // For Montgomery multiplication
    pub const RmodP: [u32; 8] = [0x00000001, 0x00000000, 0x00000000, 0xffffffff, 0xffffffff, 0xffffffff, 0xfffffffe, 0x00000000];
    // pub const RRmodP: [u32; 8] = [0x00000003, 0x00000000, 0xffffffff, 0xfffffffb, 0xfffffffe, 0xffffffff, 0xfffffffd, 0x00000004];
    pub const TWO_R_modP: [u32; 8] = [0x00000002, 0x00000000, 0x00000000, 0xfffffffe, 0xffffffff, 0xffffffff, 0xfffffffd, 0x00000001];
    pub const THREE_R_modP: [u32; 8] = [0x00000003, 0x00000000, 0x00000000, 0xfffffffd, 0xffffffff, 0xffffffff, 0xfffffffc, 0x00000002];
    pub const A_R_modP: [u32; 8] = [0xfffffffc, 0xffffffff, 0xffffffff, 0x00000003, 0x00000000, 0x00000000, 0x00000004, 0xfffffffc];

    // Curve generator, in Montgomery form
    pub const GxRmodP: [u32; 8] = [0x18a9143c, 0x79e730d4, 0x5fedb601, 0x75ba95fc, 0x77622510, 0x79fb732b, 0xa53755c6, 0x18905f76];
    pub const GyRmodP: [u32; 8] = [0xce95560a, 0xddf25357, 0xba19e45c, 0x8b4ab8e4, 0xdd21f325, 0xd2e88688, 0x25885d85, 0x8571ff18];

    pub const BARRETT_MOD_N: [u32; 9] = [0xeedf9bfe, 0x012ffd85, 0xdf1a6c21, 0x43190552, 0xffffffff, 0xfffffffe, 0xffffffff, 0x00000000, 0x1];
}

// This is an implementation of Montgomery reduction for bignums
// (multiprecision integers). The algorithm is described on Wikipedia
// at https://en.wikipedia.org/wiki/Montgomery_modular_multiplication#Montgomery_arithmetic_on_multiprecision_(variable-radix)_integers
// This implementation was originally written based off of a different reference
// and therefore uses different variable names from that article.
// The number to be reduced is in the argument inp which is called T in the article.
// The modulus to reduce by is in the argument n which is called N in the article.
// kn is an input used to work around lack of stack space in the firmware
// and is a temporary used to store what is denoted as m * N in the article.
// The output is stored back in inp (which is mutated during the computation)
// but this function will return a borrow to the appropriate slice of inp
// (e.g. the value denoted S in the article is never explicitly created
// and the shift / division by R is never explicitly performed and are
// done by reborrowing the appropriate subset of the input words)
// The base B is 2^32. The value of R is (2^32)^n.len() and so r is n.len()
// (and so is p)
fn mont_core<'a, 'b, 'c>(inp: &'a mut [u32], n: &'b [u32], kn: &'c mut [u32]) -> &'a mut [u32] {
    // Need an extra word because the intermediate step can overflow by 1 bit
    debug_assert!(inp.len() == 2 * n.len() + 1);
    debug_assert!(kn.len() == n.len() + 1);

    // least significant word of n
    let n0: u32 = n[0];
    let n0_negmodinv: u32 = (!modinv32(n0)).wrapping_add(1);
    // n0_negmodinv is called N' in the article and is the negative of the
    // modular multiplicative inverse of N mod B.

    // This loop is denoted "loop1" in the article
    for i in 0..n.len() {
        // This is T[i] in the article
        let work_word: u32 = inp[i];
        // This is m in the article
        let k: u32 = work_word.wrapping_mul(n0_negmodinv);
        // This computes m * N, but reuses the existing multiply/add
        // helper functions rather than mixing their functions into here.
        // Note that the Wikipedia loop2 loops through all of N (indexing by j)
        // and multiplies by m. This operation is performed by mulhelper.
        mulhelper(kn, &n, &[k]);
        // Note that the Wikipedia loop2 and loop3 index into T with i+j.
        // The indexing by j is taken care of by addhelper, and the offset of
        // i is implemented by slicing.
        let carry = addhelper(&mut inp[i..], &kn);
        // Because we have allocated an entire extra carry word, there should
        // never be a carry out of the add function.
        debug_assert!(!carry);
    }

    // This ensures that the result at this point is indeed divisible by R
    for i in 0..n.len() {
        debug_assert!(inp[i] == 0);
    }

    // This divides by R by reborrowing the appropriate words. Note that
    // the extra carry word is still included.
    let reduced_inp = &mut inp[n.len()..];

    // If still >= n then need to subtract n
    if cmphelper(reduced_inp, n) != ::core::cmp::Ordering::Less {
        let borrow = subhelper(reduced_inp, n);
        debug_assert!(!borrow);
    }

    // At this point, the carry word should always be 0 because the result
    // should be less than N (which doesn't have an extra word).
    debug_assert!(reduced_inp[reduced_inp.len() - 1] == 0);
    let final_len = reduced_inp.len() - 1;
    // Return the correct slice, cutting off the carry word
    &mut reduced_inp[..final_len]
}

// Converts a out of Montgomery representation by running the reduction
// algorithm (which divides by R)
pub fn mont_red(a: &Bignum2048, n: &Bignum2048,
    work_bignum_4096_oversized: &mut Bignum4096Oversized,
    work_bignum_2048_oversized: &mut Bignum2048Oversized) -> Bignum2048 {

    let x = work_bignum_4096_oversized;
    x.0[..64].copy_from_slice(&a.0);
    for i in 64..x.0.len() {
        x.0[i] = 0;
    }
    let work_kn = work_bignum_2048_oversized;
    let result = mont_core(&mut x.0, &n.0, &mut work_kn.0);
    let mut ret: Bignum2048 = Bignum2048([0; 64]);
    ret.0.copy_from_slice(result);
    ret
}

// Perform the RSA operation of m^e mod n using exponentiation by squaring.
// Unlike typical implementations, requires the caller to manually compute
// R mod n and R^2 mod n (where R = 2^2048). In the actual firmware, this is
// precomputed externally using a Python script with native bignum capabilities
// rather than having to implement a general-purpose modular reduction
// algorithm in this code.
pub fn rsa_pubkey_op(m: &Bignum2048, n: &Bignum2048, r: &Bignum2048, rr: &Bignum2048, mut e: u32,
    work_bignum_4096_oversized: &mut Bignum4096Oversized,
    work_bignum_2048_oversized: &mut Bignum2048Oversized,
    work_bignum_2048: &mut Bignum2048,
    work_bignum_2048_2: &mut Bignum2048) -> Bignum2048 {

    let x = work_bignum_2048;
    x.0 = r.0;     // 1 in Montgomery form
    // Bring m into Montgomery form by multiplying by R^2 and performing
    // Montgomery reduction
    *work_bignum_2048_2 = mont_mul(m, rr, n, work_bignum_4096_oversized, work_bignum_2048_oversized);
    let m = work_bignum_2048_2;

    // Exponentiation by squaring
    while e > 0 {
        if e & 1 != 0 {
            *x = mont_mul(&x, &m, n, work_bignum_4096_oversized, work_bignum_2048_oversized);
            e = e - 1;
        }
        if e != 0 {
            *x = mont_mul(&x, &x, n, work_bignum_4096_oversized, work_bignum_2048_oversized);
        }
        e = e / 2;
    }

    // Finally, take the output out of Montgomery form
    mont_red(&x, n, work_bignum_4096_oversized, work_bignum_2048_oversized)
}

// This reduces x mod n using the Barrett reduction algorithm. It is written
// based on the article at https://www.nayuki.io/page/barrett-reduction-algorithm
// and uses its notation. The caller must precompute and pass in the precomputed
// factor denoted r in the article. This is precomputed and hardcoded in this
// ECDSA implementation.
fn barrett_reduction_core(x: &mut [u32], r: &[u32], n: &[u32],
    scratch_xr: &mut [u32], scratch2: &mut [u32]) {

    // n is N bits, x is 2N bits, r is N+1 bits
    // scratch_xr should be 3N bits but due to lazy is 3N+1
    // scratch2 should be 2N bits
    // Bit width analysis taken from the article
    debug_assert!(x.len() == 2 * n.len());
    debug_assert!(r.len() == n.len() + 1);
    debug_assert!(scratch_xr.len() == 3 * n.len() + 1);
    debug_assert!(scratch2.len() == 2 * n.len());

    // Compute x * r
    mulhelper(scratch_xr, x, r);
    debug_assert!(scratch_xr[scratch_xr.len() - 1] == 0);
    let xr_div4k = &scratch_xr[2 * n.len()..3 * n.len()];

    // Multiply n * (xr / 4^k)
    mulhelper(scratch2, xr_div4k, n);

    // Subtract this from x
    let borrow = subhelper(x, scratch2);
    debug_assert!(!borrow);

    // If still >= n then need to subtract n
    if cmphelper(x, n) != ::core::cmp::Ordering::Less {
        let borrow = subhelper(x, n);
        debug_assert!(!borrow);
    }

    // Should be done now
    for i in n.len()..x.len() {
        debug_assert!(x[i] == 0);
    }
}

// Find the modular multiplicative inverse of x mod p using the extended
// binary GCD algorithm (Stein's algorithm)
// FIXME: Where did this come from?
fn inverse_mod_core(x: &[u32], p: &[u32], out: &mut [u32],
    scratch0: &mut [u32], scratch1: &mut [u32], scratch2: &mut [u32], scratch3: &mut [u32]) {

    debug_assert!(x.len() == p.len());
    debug_assert!(out.len() == p.len());
    debug_assert!(x.len() + 1 == scratch0.len());
    debug_assert!(x.len() + 1 == scratch1.len());
    debug_assert!(x.len() + 1 == scratch2.len());
    debug_assert!(x.len() + 1 == scratch3.len());

    let lenlen = scratch0.len();

    let u = scratch0;
    let v = scratch1;
    let b = scratch2;
    let d = scratch3;

    u[..lenlen - 1].copy_from_slice(p);
    u[lenlen - 1] = 0;
    v[..lenlen - 1].copy_from_slice(x);
    v[lenlen - 1] = 0;
    for i in 0..lenlen {
        b[i] = 0;
    }
    for i in 0..lenlen {
        d[i] = 0;
    }
    d[0] = 1;

    while !iszero(u) {
        while u[0] & 1 == 0 {
            shr1helper(u);
            if b[0] & 1 == 0 {
                shr1helper(b);
            } else {
                subhelper(b, p);
                shr1helper(b);
            }
        }
        while v[0] & 1 == 0 {
            shr1helper(v);
            if d[0] & 1 == 0 {
                shr1helper(d);
            } else {
                subhelper(d, p);
                shr1helper(d);
            }
        }
        if cmphelper(u, v) != ::core::cmp::Ordering::Less {
            subhelper(u, v);
            subhelper(b, d);
        } else {
            subhelper(v, u);
            subhelper(d, b);
        }
    }

    if d[lenlen - 1] & 0x80000000 != 0 {
        // Negative
        addhelper(d, p);
    }
    debug_assert!(d[lenlen - 1] == 0);
    out.copy_from_slice(&d[..lenlen - 1]);
}

pub struct Bignum256(pub [u32; 8]);
pub struct Bignum256Oversized(pub [u32; 9]);
pub struct Bignum512Oversized(pub [u32; 17]);
pub struct ECPointAffine256{pub x: Bignum256, pub y: Bignum256}
pub struct ECPointProjective256{pub x: Bignum256, pub y: Bignum256, pub z: Bignum256}

// Return a * b mod n using Montgomery multiplication
fn mont_mul_256(a: &Bignum256, b: &Bignum256, n: &Bignum256) -> Bignum256 {
    let mut x: Bignum512Oversized = Bignum512Oversized([0; 17]);
    mulhelper(&mut x.0[..16], &a.0, &b.0);
    let mut work_kn: Bignum256Oversized = Bignum256Oversized([0; 9]);
    let result = mont_core(&mut x.0, &n.0, &mut work_kn.0);
    let mut ret: Bignum256 = Bignum256([0; 8]);
    ret.0.copy_from_slice(result);
    ret
}

// Converts a out of Montgomery representation by running the reduction
// algorithm (which divides by R)
fn mont_red_256(a: &Bignum256, n: &Bignum256) -> Bignum256 {
    let mut x: Bignum512Oversized = Bignum512Oversized([0; 17]);
    x.0[..8].copy_from_slice(&a.0);
    let mut work_kn: Bignum256Oversized = Bignum256Oversized([0; 9]);
    let result = mont_core(&mut x.0, &n.0, &mut work_kn.0);
    let mut ret: Bignum256 = Bignum256([0; 8]);
    ret.0.copy_from_slice(result);
    ret
}

// Return a * b mod n using Montgomery multiplication
fn mont_mul_256_oversized(a: &Bignum256, b: &Bignum256, n: &Bignum256) -> Bignum256Oversized {
    let mut x: Bignum512Oversized = Bignum512Oversized([0; 17]);
    mulhelper(&mut x.0[..16], &a.0, &b.0);
    let mut work_kn: Bignum256Oversized = Bignum256Oversized([0; 9]);
    let result = mont_core(&mut x.0, &n.0, &mut work_kn.0);
    let mut ret: Bignum256Oversized = Bignum256Oversized([0; 9]);
    ret.0[..8].copy_from_slice(result);
    ret
}

// Computes a + b mod n
fn addmodn256(a: &mut Bignum256Oversized, b: &Bignum256, n: &Bignum256) {
    debug_assert!(a.0[8] == 0);

    addhelper(&mut a.0, &b.0);

    if cmphelper(&a.0, &n.0) != ::core::cmp::Ordering::Less {
        let borrow = subhelper(&mut a.0, &n.0);
        debug_assert!(!borrow);
    }
}

// Computes a - b mod n
fn submodn256(a: &mut Bignum256Oversized, b: &Bignum256, n: &Bignum256) {
    debug_assert!(a.0[8] == 0);

    let borrow = subhelper(&mut a.0, &b.0);

    if borrow {
        let carry = addhelper(&mut a.0, &n.0);
        debug_assert!(carry);
    }
}

// Compute a point doubling in projective coordinates. Formulas taken from
// https://www.nayuki.io/page/elliptic-curve-point-addition-in-projective-coordinates
// and uses its notation. All operations are performed in Montgomery coordinates
// and requires precomputing the Montgomery form of constants like 2 and 3.
pub fn point_double_projective(x: &ECPointProjective256) -> ECPointProjective256 {
    // In general, "oversize" outputs are used where necessary to allow for
    // a carry word for addition/subtraction before reduction by the modulus

    // 3X^2
    let mut t1 = mont_mul_256_oversized(&Bignum256(secp256r1::THREE_R_modP),
        &mont_mul_256(&x.x, &x.x, &Bignum256(secp256r1::P)), &Bignum256(secp256r1::P));
    // aZ^2
    let t2 = mont_mul_256(&Bignum256(secp256r1::A_R_modP),
        &mont_mul_256(&x.z, &x.z, &Bignum256(secp256r1::P)), &Bignum256(secp256r1::P));
    // T = 3X^2 + aZ^2
    addmodn256(&mut t1, &t2, &Bignum256(secp256r1::P));
    let t = Bignum256([t1.0[0], t1.0[1], t1.0[2], t1.0[3], t1.0[4], t1.0[5], t1.0[6], t1.0[7]]);

    // U = 2XY
    let u = mont_mul_256(&Bignum256(secp256r1::TWO_R_modP),
        &mont_mul_256(&x.y, &x.z, &Bignum256(secp256r1::P)), &Bignum256(secp256r1::P));

    // V = 2UXY
    // Will be reused later for a different computation
    let mut vext = mont_mul_256_oversized(
        &mont_mul_256(&Bignum256(secp256r1::TWO_R_modP), &u, &Bignum256(secp256r1::P)),
        &mont_mul_256(&x.x, &x.y, &Bignum256(secp256r1::P)), &Bignum256(secp256r1::P));
    let v = Bignum256([vext.0[0], vext.0[1], vext.0[2], vext.0[3],
        vext.0[4], vext.0[5], vext.0[6], vext.0[7]]);

    // T^2
    let mut w1 = mont_mul_256_oversized(&t, &t, &Bignum256(secp256r1::P));
    // 2V
    let w2 = mont_mul_256(&Bignum256(secp256r1::TWO_R_modP), &v, &Bignum256(secp256r1::P));
    // W = T^2 - 2V
    submodn256(&mut w1, &w2, &Bignum256(secp256r1::P));
    let w = Bignum256([w1.0[0], w1.0[1], w1.0[2], w1.0[3], w1.0[4], w1.0[5], w1.0[6], w1.0[7]]);

    // Xout = UW
    let xout = mont_mul_256(&u, &w, &Bignum256(secp256r1::P));
    // Zout = U^3
    let zout = mont_mul_256(&u,
        &mont_mul_256(&u, &u, &Bignum256(secp256r1::P)), &Bignum256(secp256r1::P));

    // UY
    let uy0 = mont_mul_256(&u, &x.y, &Bignum256(secp256r1::P));
    // V - W
    let vminusw = &mut vext;
    submodn256(vminusw, &w, &Bignum256(secp256r1::P));
    let vminusw = Bignum256([vminusw.0[0], vminusw.0[1], vminusw.0[2], vminusw.0[3],
        vminusw.0[4], vminusw.0[5], vminusw.0[6], vminusw.0[7]]);
    // T(V - W)
    let mut y1 = mont_mul_256_oversized(&t, &vminusw, &Bignum256(secp256r1::P));
    // 2(UY)^2
    let y2 = mont_mul_256(&Bignum256(secp256r1::TWO_R_modP),
        &mont_mul_256(&uy0, &uy0, &Bignum256(secp256r1::P)), &Bignum256(secp256r1::P));
    // Yout = T(V - W) - 2(UY)^2
    submodn256(&mut y1, &y2, &Bignum256(secp256r1::P));
    let yout = Bignum256([y1.0[0], y1.0[1], y1.0[2], y1.0[3], y1.0[4], y1.0[5], y1.0[6], y1.0[7]]);

    ECPointProjective256{x: xout, y: yout, z: zout}
}

// Compute a point addition between a point in projective coordinates and a
// point in affine coordinates. Uses notation from the same article.
pub fn point_add_projective_affine(p1: &ECPointProjective256, p2: &ECPointAffine256)
    -> ECPointProjective256 {

    // The projective coordinate point is the only one that can represent
    // "the point at infinity" O. If p1 is currently O, the result is p2.
    // However, p2 needs to be converted to projective coordinates by
    // setting the Z value to 1 (which is represented by R because all numbers
    // are in Montgomery form).
    if iszero(&p1.z.0) {
        return ECPointProjective256 {
            x: Bignum256(p2.x.0),
            y: Bignum256(p2.y.0),
            z: Bignum256(secp256r1::RmodP),
        }
    }
    // p2 cannot be O

    let x0 = &p1.x;
    let y0 = &p1.y;
    let z0 = &p1.z;
    let x1 = &p2.x;
    let y1 = &p2.y;
    // z1 is known to always be 1 because p2 uses affine coordinates

    // T0 = Y0 Z1 = Y0
    let mut t0 = Bignum256Oversized([0; 9]);
    t0.0[..8].copy_from_slice(&y0.0);
    // T1 = Y1 Z0
    let t1 = mont_mul_256(y1, z0, &Bignum256(secp256r1::P));
    // T = T0 - T1
    submodn256(&mut t0, &t1, &Bignum256(secp256r1::P));
    let t = Bignum256([t0.0[0], t0.0[1], t0.0[2], t0.0[3], t0.0[4], t0.0[5], t0.0[6], t0.0[7]]);

    // U0 = X0 Z1 = X0
    // Two copies are made because an additional temporary is needed later
    let mut u0 = Bignum256Oversized([0; 9]);
    u0.0[..8].copy_from_slice(&x0.0);
    let mut u0_ = Bignum256Oversized([0; 9]);
    u0_.0[..8].copy_from_slice(&x0.0);
    // U1 = X1 Z0
    let u1 = mont_mul_256(x1, z0, &Bignum256(secp256r1::P));
    // U = U0 - U1
    submodn256(&mut u0, &u1, &Bignum256(secp256r1::P));
    let u = Bignum256([u0.0[0], u0.0[1], u0.0[2], u0.0[3], u0.0[4], u0.0[5], u0.0[6], u0.0[7]]);

    // T = 0 and U = 0 implies T0 = T1 and U0 = U1. This then implies
    // Y0 Z1 = Y1 Z0 and X0 Z1 = X1 Z0 which then implies that
    // Y0/Z0 = Y1/Z1 and X0/Z0 = X1/X1 which means that the two input points
    // have identical x and y coordinates. This is a point doubling and the
    // special formula has to be used.
    if iszero(&t.0) && iszero(&u.0) {
        return point_double_projective(p1);
    }

    // T is not zero but U is zero which means that the y coordinates of the
    // points are not equal but the x coordinates are equal. This means that
    // the points are inverses and the result must be the point at infinity.
    if iszero(&u.0) {
        return ECPointProjective256 {
            x: Bignum256([0; 8]),
            y: Bignum256([0; 8]),
            z: Bignum256([0; 8]),
        }
    }

    // Now do the point addition for real

    // U2 = U^2
    let u2 = mont_mul_256(&u, &u, &Bignum256(secp256r1::P));
    // U3 = U U2
    let u3 = mont_mul_256(&u, &u2, &Bignum256(secp256r1::P));

    // T^2 V = T^2 Z0 Z1 = T^2 Z0
    let mut w1 = mont_mul_256_oversized(
        &mont_mul_256(&t, &t, &Bignum256(secp256r1::P)),
        z0, &Bignum256(secp256r1::P));
    // U0 + U1
    addmodn256(&mut u0_, &u1, &Bignum256(secp256r1::P));
    let u0pu1 = Bignum256([u0_.0[0], u0_.0[1], u0_.0[2], u0_.0[3],
        u0_.0[4], u0_.0[5], u0_.0[6], u0_.0[7]]);
    // U2(U0 + U1)
    let w2 = mont_mul_256(&u2, &u0pu1, &Bignum256(secp256r1::P));
    // W = T^2 Z0 - U2(U0 + U1)
    submodn256(&mut w1, &w2, &Bignum256(secp256r1::P));
    let w = Bignum256([w1.0[0], w1.0[1], w1.0[2], w1.0[3], w1.0[4], w1.0[5], w1.0[6], w1.0[7]]);

    // Xout = U W
    let xout = mont_mul_256(&u, &w, &Bignum256(secp256r1::P));
    // Zout = U3 V = U3 Z0
    let zout = mont_mul_256(&u3, z0, &Bignum256(secp256r1::P));

    // U0 U2
    let mut u0u2 = mont_mul_256_oversized(x0, &u2, &Bignum256(secp256r1::P));
    // U0 U2 - W
    submodn256(&mut u0u2, &w, &Bignum256(secp256r1::P));
    let u0u2minusw = Bignum256([u0u2.0[0], u0u2.0[1], u0u2.0[2], u0u2.0[3],
        u0u2.0[4], u0u2.0[5], u0u2.0[6], u0u2.0[7]]);
    // T(U0 U2 - W)
    let mut yout1 = mont_mul_256_oversized(&u0u2minusw, &t, &Bignum256(secp256r1::P));
    // T0 U3 = Y0 U3 (the t0 variable has been mutated to the value T)
    let yout2 = mont_mul_256(y0, &u3, &Bignum256(secp256r1::P));
    // Yout = T(U0 U2 - W) - T0 U3
    submodn256(&mut yout1, &yout2, &Bignum256(secp256r1::P));
    let yout = Bignum256([yout1.0[0], yout1.0[1], yout1.0[2], yout1.0[3],
        yout1.0[4], yout1.0[5], yout1.0[6], yout1.0[7]]);

    ECPointProjective256 {
        x: xout,
        y: yout,
        z: zout,
    }
}

// Compute k1 * pubk + k2 * generator using the double-and-add algorithm.
// This uses the Straus-Shamir trick to move the final addition inside the
// inner loop. Note that all input points must still be in Montgomery form.
pub fn ec_two_scalar_mult_shamir(k1: &Bignum256, k2: &Bignum256,
    pubk: &ECPointAffine256, pubk_plus_generator: &ECPointAffine256) -> ECPointProjective256 {

    let mut x = ECPointProjective256 {
        x: Bignum256([0; 8]),
        y: Bignum256([0; 8]),
        z: Bignum256([0; 8]),
    };

    for word_i in (0..8).rev() {
        for bit_i in (0..32).rev() {
            let bit1 = k1.0[word_i] & (1 << bit_i) != 0;
            let bit2 = k2.0[word_i] & (1 << bit_i) != 0;

            if bit1 && bit2 {
                // P + Q
                x = point_add_projective_affine(&x, pubk_plus_generator);
            } else if bit1 {
                // P
                x = point_add_projective_affine(&x, pubk);
            } else if bit2 {
                // Q
                x = point_add_projective_affine(&x,
                    &ECPointAffine256{
                        x: Bignum256(secp256r1::GxRmodP),
                        y: Bignum256(secp256r1::GyRmodP),
                    });
            }

            if !(word_i == 0 && bit_i == 0) {
                x = point_double_projective(&x);
            }
        }
    }

    x
}

// Verify an ECDSA signature. Note that the public key must be in Montgomery
// form. This is implemented based on the notation in the Wikipedia article
// https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm#Signature_verification_algorithm
pub fn ecdsa_secp256r1_verify_sig(pubk: &ECPointAffine256, pubk_plus_generator: &ECPointAffine256,
    hash: &Bignum256, sig_r: &Bignum256, sig_s: &Bignum256) -> bool {

    // Does not verify that the public key is indeed a valid curve point.
    // The intended use case has the key hardcoded in the binary (so hopefully
    // the key is indeed valid).

    // Verify that r and s are in the correct range
    if iszero(&sig_r.0) {
        return false;
    }
    if iszero(&sig_s.0) {
        return false;
    }
    if cmphelper(&sig_r.0, &secp256r1::N) != ::core::cmp::Ordering::Less {
        return false;
    }
    if cmphelper(&sig_s.0, &secp256r1::N) != ::core::cmp::Ordering::Less {
        return false;
    }

    // w is s^{-1}
    let mut w = [0u32; 8];
    {
        // Scope is to force scratch variables to be dropped to save stack space
        let mut scratch0 = [0u32; 9];
        let mut scratch1 = [0u32; 9];
        let mut scratch2 = [0u32; 9];
        let mut scratch3 = [0u32; 9];
        inverse_mod_core(&sig_s.0, &secp256r1::N, &mut w,
            &mut scratch0, &mut scratch1, &mut scratch2, &mut scratch3);
    }

    let mut u1 = [0u32; 8];
    let mut u2 = [0u32; 8];
    {
        let mut barrett_scratch_xr = [0u32; 25];
        let mut barrett_scratch2 = [0u32; 16];
        let mut outtmp = [0u32; 16];

        // u1 = zs^{-1} mod n
        mulhelper(&mut outtmp, &hash.0, &w);
        barrett_reduction_core(&mut outtmp, &secp256r1::BARRETT_MOD_N, &secp256r1::N,
            &mut barrett_scratch_xr, &mut barrett_scratch2);
        u1.copy_from_slice(&outtmp[..8]);
        // u2 = rs^{-1} mod n
        mulhelper(&mut outtmp, &sig_r.0, &w);
        barrett_reduction_core(&mut outtmp, &secp256r1::BARRETT_MOD_N, &secp256r1::N,
            &mut barrett_scratch_xr, &mut barrett_scratch2);
        u2.copy_from_slice(&outtmp[..8]);
    }

    // Resulting curve point, but it's still in projective coordinates
    let curve_op_result = ec_two_scalar_mult_shamir(&Bignum256(u2), &Bignum256(u1),
        pubk, pubk_plus_generator);

    if iszero(&curve_op_result.z.0) {
        // Cannot be O here
        return false;
    }

    // Calculate Z in not-Montgomery form
    let result_z_not_mont = mont_red_256(&curve_op_result.z, &Bignum256(secp256r1::P));
    // Calculate 1/Z, still in not-Montgomery form
    let mut one_over_z = [0u32; 8];
    {
        let mut scratch0 = [0u32; 9];
        let mut scratch1 = [0u32; 9];
        let mut scratch2 = [0u32; 9];
        let mut scratch3 = [0u32; 9];
        inverse_mod_core(&result_z_not_mont.0, &secp256r1::P, &mut one_over_z,
            &mut scratch0, &mut scratch1, &mut scratch2, &mut scratch3);
    }
    // Compute X/Z. Because 1/Z is not in Montgomery form but X still is, the
    // Montgomery multiplication algorithm will automatically remove the
    // remaining factor of R and yield a result in not-Montgomery form.
    let result_x = mont_mul_256(
        &curve_op_result.x, &Bignum256(one_over_z), &Bignum256(secp256r1::P));

    // The X value needs to be reduced mod n (the above curve coordinate math
    // was done mod p)
    let mut result_x_reduced = [0u32; 8];
    {
        let mut barrett_scratch_xr = [0u32; 25];
        let mut barrett_scratch2 = [0u32; 16];
        let mut outtmp = [0u32; 16];
        outtmp[..8].copy_from_slice(&result_x.0);

        barrett_reduction_core(&mut outtmp, &secp256r1::BARRETT_MOD_N, &secp256r1::N,
            &mut barrett_scratch_xr, &mut barrett_scratch2);
        result_x_reduced.copy_from_slice(&outtmp[..8]);
    }

    // finally check x == r
    cmphelper(&result_x_reduced, &sig_r.0) == ::core::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use *;

    fn mul2048(a: &Bignum2048, b: &Bignum2048) -> Bignum4096Oversized {
        let mut ret = Bignum4096Oversized([0; 129]);
        mulhelper(&mut ret.0[..128], &a.0, &b.0);
        ret
    }

    #[test]
    fn test_rsaop() {
        let mut n = Bignum2048([0; 64]);
        let mut r = Bignum2048([0; 64]);
        let mut rr = Bignum2048([0; 64]);
        let mut m = Bignum2048([0; 64]);
        let mut expected = Bignum2048([0; 64]);

        let mut work_2048 = Bignum2048([0; 64]);
        let mut work_2048_2 = Bignum2048([0; 64]);
        let mut work_2048o = Bignum2048Oversized([0; 65]);
        let mut work_4096o = Bignum4096Oversized([0; 129]);

        n.0[63] = 0xd2fc5273;
        n.0[62] = 0xa2f9262b;
        n.0[61] = 0x84863bf6;
        n.0[60] = 0xe5b8fdd9;
        n.0[59] = 0x964e6190;
        n.0[58] = 0x2204cd46;
        n.0[57] = 0x46a16cac;
        n.0[56] = 0xca2d46cb;
        n.0[55] = 0x9a9ae44e;
        n.0[54] = 0xb1c387b6;
        n.0[53] = 0x395fee7e;
        n.0[52] = 0xa32cf051;
        n.0[51] = 0x8ff4bd02;
        n.0[50] = 0xe0b52ee9;
        n.0[49] = 0xa368fcc6;
        n.0[48] = 0xb8111d2b;
        n.0[47] = 0xaa1ad23b;
        n.0[46] = 0xfc741046;
        n.0[45] = 0x9701f53e;
        n.0[44] = 0x795d4599;
        n.0[43] = 0x01262d17;
        n.0[42] = 0x290c43a4;
        n.0[41] = 0x84faaef4;
        n.0[40] = 0xcfea8484;
        n.0[39] = 0x655231ec;
        n.0[38] = 0xd9bd5065;
        n.0[37] = 0xc35839ef;
        n.0[36] = 0x0b8dea96;
        n.0[35] = 0xe51bf71b;
        n.0[34] = 0xdcb821e8;
        n.0[33] = 0x57c3b561;
        n.0[32] = 0xf0ea71d9;
        n.0[31] = 0x57ac2755;
        n.0[30] = 0x4e1abc59;
        n.0[29] = 0x75897241;
        n.0[28] = 0x4c36c21b;
        n.0[27] = 0x6a402e4e;
        n.0[26] = 0x91b37aa9;
        n.0[25] = 0xf608ca52;
        n.0[24] = 0x9f60173d;
        n.0[23] = 0x8213f7fa;
        n.0[22] = 0x97ad73fa;
        n.0[21] = 0xab488831;
        n.0[20] = 0x757dc4b4;
        n.0[19] = 0x8f748958;
        n.0[18] = 0x2050589f;
        n.0[17] = 0xe265a540;
        n.0[16] = 0x526a7b3b;
        n.0[15] = 0xb2fa065c;
        n.0[14] = 0x0654f4b7;
        n.0[13] = 0x459bea90;
        n.0[12] = 0x544bae40;
        n.0[11] = 0x80da5f73;
        n.0[10] = 0x1e79b4d0;
        n.0[ 9] = 0x18426fdc;
        n.0[ 8] = 0xe7f15982;
        n.0[ 7] = 0x1ca1d21d;
        n.0[ 6] = 0xe3abf69b;
        n.0[ 5] = 0x738dd481;
        n.0[ 4] = 0x7f8e60de;
        n.0[ 3] = 0xe446e48c;
        n.0[ 2] = 0x8a986adb;
        n.0[ 1] = 0xfc4dbfe1;
        n.0[ 0] = 0xa3cec7a5;

        r.0[63] = 0x2d03ad8c;
        r.0[62] = 0x5d06d9d4;
        r.0[61] = 0x7b79c409;
        r.0[60] = 0x1a470226;
        r.0[59] = 0x69b19e6f;
        r.0[58] = 0xddfb32b9;
        r.0[57] = 0xb95e9353;
        r.0[56] = 0x35d2b934;
        r.0[55] = 0x65651bb1;
        r.0[54] = 0x4e3c7849;
        r.0[53] = 0xc6a01181;
        r.0[52] = 0x5cd30fae;
        r.0[51] = 0x700b42fd;
        r.0[50] = 0x1f4ad116;
        r.0[49] = 0x5c970339;
        r.0[48] = 0x47eee2d4;
        r.0[47] = 0x55e52dc4;
        r.0[46] = 0x038befb9;
        r.0[45] = 0x68fe0ac1;
        r.0[44] = 0x86a2ba66;
        r.0[43] = 0xfed9d2e8;
        r.0[42] = 0xd6f3bc5b;
        r.0[41] = 0x7b05510b;
        r.0[40] = 0x30157b7b;
        r.0[39] = 0x9aadce13;
        r.0[38] = 0x2642af9a;
        r.0[37] = 0x3ca7c610;
        r.0[36] = 0xf4721569;
        r.0[35] = 0x1ae408e4;
        r.0[34] = 0x2347de17;
        r.0[33] = 0xa83c4a9e;
        r.0[32] = 0x0f158e26;
        r.0[31] = 0xa853d8aa;
        r.0[30] = 0xb1e543a6;
        r.0[29] = 0x8a768dbe;
        r.0[28] = 0xb3c93de4;
        r.0[27] = 0x95bfd1b1;
        r.0[26] = 0x6e4c8556;
        r.0[25] = 0x09f735ad;
        r.0[24] = 0x609fe8c2;
        r.0[23] = 0x7dec0805;
        r.0[22] = 0x68528c05;
        r.0[21] = 0x54b777ce;
        r.0[20] = 0x8a823b4b;
        r.0[19] = 0x708b76a7;
        r.0[18] = 0xdfafa760;
        r.0[17] = 0x1d9a5abf;
        r.0[16] = 0xad9584c4;
        r.0[15] = 0x4d05f9a3;
        r.0[14] = 0xf9ab0b48;
        r.0[13] = 0xba64156f;
        r.0[12] = 0xabb451bf;
        r.0[11] = 0x7f25a08c;
        r.0[10] = 0xe1864b2f;
        r.0[ 9] = 0xe7bd9023;
        r.0[ 8] = 0x180ea67d;
        r.0[ 7] = 0xe35e2de2;
        r.0[ 6] = 0x1c540964;
        r.0[ 5] = 0x8c722b7e;
        r.0[ 4] = 0x80719f21;
        r.0[ 3] = 0x1bb91b73;
        r.0[ 2] = 0x75679524;
        r.0[ 1] = 0x03b2401e;
        r.0[ 0] = 0x5c31385b;

        rr.0[63] = 0xa6a8b5f9;
        rr.0[62] = 0xfc683183;
        rr.0[61] = 0x660732ec;
        rr.0[60] = 0x7fad3138;
        rr.0[59] = 0x74d59bd3;
        rr.0[58] = 0x8cc3374a;
        rr.0[57] = 0x924f6690;
        rr.0[56] = 0xde3a3b5b;
        rr.0[55] = 0x8f8a7a64;
        rr.0[54] = 0x2c099f91;
        rr.0[53] = 0x33a45313;
        rr.0[52] = 0xf4a30ae3;
        rr.0[51] = 0x1fd6b74c;
        rr.0[50] = 0x1e5142af;
        rr.0[49] = 0x1feadbee;
        rr.0[48] = 0xa948a5b7;
        rr.0[47] = 0x7ffc2af3;
        rr.0[46] = 0x7c9bfd18;
        rr.0[45] = 0x6f00c434;
        rr.0[44] = 0xf489b5c9;
        rr.0[43] = 0x08850474;
        rr.0[42] = 0xc0e56809;
        rr.0[41] = 0x8ddef55d;
        rr.0[40] = 0xd0dc4f59;
        rr.0[39] = 0x08eb029e;
        rr.0[38] = 0x10f5315c;
        rr.0[37] = 0xae0ed8a3;
        rr.0[36] = 0x406dc80a;
        rr.0[35] = 0xe9406846;
        rr.0[34] = 0x6f10a46c;
        rr.0[33] = 0x9a741f79;
        rr.0[32] = 0x5d3feb16;
        rr.0[31] = 0x010023d9;
        rr.0[30] = 0x84de714b;
        rr.0[29] = 0x0943a775;
        rr.0[28] = 0x6f052d38;
        rr.0[27] = 0x5c36b89e;
        rr.0[26] = 0x6e6927c5;
        rr.0[25] = 0xcce5c6ff;
        rr.0[24] = 0x5aa1335a;
        rr.0[23] = 0x89b543e7;
        rr.0[22] = 0x02a9b47a;
        rr.0[21] = 0x00213bb7;
        rr.0[20] = 0x57469a06;
        rr.0[19] = 0xaeb9d46a;
        rr.0[18] = 0xdadf900a;
        rr.0[17] = 0xd209bb8a;
        rr.0[16] = 0x813c6fff;
        rr.0[15] = 0xb8e55d10;
        rr.0[14] = 0x7857c562;
        rr.0[13] = 0x78d195b1;
        rr.0[12] = 0xdabd2ea8;
        rr.0[11] = 0x6d72be73;
        rr.0[10] = 0xf01e2f77;
        rr.0[ 9] = 0x261925c3;
        rr.0[ 8] = 0xab87c57b;
        rr.0[ 7] = 0xe93885d2;
        rr.0[ 6] = 0x85400f34;
        rr.0[ 5] = 0xd83910ab;
        rr.0[ 4] = 0x687a0524;
        rr.0[ 3] = 0xcc9af25b;
        rr.0[ 2] = 0x2537f2f4;
        rr.0[ 1] = 0xc381e66e;
        rr.0[ 0] = 0x761158e0;

        m.0[63] = 0x4550c186;
        m.0[62] = 0x7d0ec1b8;
        m.0[61] = 0x76e96174;
        m.0[60] = 0x4bfad3d3;
        m.0[59] = 0x529e665d;
        m.0[58] = 0x5dd8921e;
        m.0[57] = 0xb9b43d67;
        m.0[56] = 0xf489d402;
        m.0[55] = 0xca39d95a;
        m.0[54] = 0xa9595c14;
        m.0[53] = 0xd2aa0545;
        m.0[52] = 0xdf440b1c;
        m.0[51] = 0x1e0799d9;
        m.0[50] = 0x392c5bee;
        m.0[49] = 0x225ae8f3;
        m.0[48] = 0xf6e54455;
        m.0[47] = 0x7218fcca;
        m.0[46] = 0x6a0c1bd1;
        m.0[45] = 0x789b8124;
        m.0[44] = 0x1328436d;
        m.0[43] = 0x567d76e0;
        m.0[42] = 0xe2d3c1a2;
        m.0[41] = 0xe0785272;
        m.0[40] = 0xa26727ef;
        m.0[39] = 0xcca90453;
        m.0[38] = 0x099d36b9;
        m.0[37] = 0xcde87ef5;
        m.0[36] = 0xb8c1d522;
        m.0[35] = 0xe35e8e56;
        m.0[34] = 0x974e5084;
        m.0[33] = 0xdd42967c;
        m.0[32] = 0x8c0d54ff;
        m.0[31] = 0xb87ae6c7;
        m.0[30] = 0x9c7501b8;
        m.0[29] = 0x1503578c;
        m.0[28] = 0x796d0579;
        m.0[27] = 0xd1c26067;
        m.0[26] = 0x8f468122;
        m.0[25] = 0x8c158fd4;
        m.0[24] = 0xe64bf422;
        m.0[23] = 0xcab8b25a;
        m.0[22] = 0x87c3d76f;
        m.0[21] = 0x394c226a;
        m.0[20] = 0xa63332dd;
        m.0[19] = 0xba1884f7;
        m.0[18] = 0xe05ffbf4;
        m.0[17] = 0xde3b613e;
        m.0[16] = 0xca6325bd;
        m.0[15] = 0x060d24c9;
        m.0[14] = 0x023315b4;
        m.0[13] = 0x69bdd42f;
        m.0[12] = 0x8abf6a76;
        m.0[11] = 0x278fb6ce;
        m.0[10] = 0x00ee970f;
        m.0[ 9] = 0x324c6666;
        m.0[ 8] = 0x15747cd1;
        m.0[ 7] = 0xe94e1121;
        m.0[ 6] = 0xf90a2d3f;
        m.0[ 5] = 0x61303967;
        m.0[ 4] = 0x855fda2d;
        m.0[ 3] = 0x06901735;
        m.0[ 2] = 0xa0f1f2cc;
        m.0[ 1] = 0x0aa5a665;
        m.0[ 0] = 0xc6a6909b;

        expected.0[63] = 0x0001ffff;
        expected.0[62] = 0xffffffff;
        expected.0[61] = 0xffffffff;
        expected.0[60] = 0xffffffff;
        expected.0[59] = 0xffffffff;
        expected.0[58] = 0xffffffff;
        expected.0[57] = 0xffffffff;
        expected.0[56] = 0xffffffff;
        expected.0[55] = 0xffffffff;
        expected.0[54] = 0xffffffff;
        expected.0[53] = 0xffffffff;
        expected.0[52] = 0xffffffff;
        expected.0[51] = 0xffffffff;
        expected.0[50] = 0xffffffff;
        expected.0[49] = 0xffffffff;
        expected.0[48] = 0xffffffff;
        expected.0[47] = 0xffffffff;
        expected.0[46] = 0xffffffff;
        expected.0[45] = 0xffffffff;
        expected.0[44] = 0xffffffff;
        expected.0[43] = 0xffffffff;
        expected.0[42] = 0xffffffff;
        expected.0[41] = 0xffffffff;
        expected.0[40] = 0xffffffff;
        expected.0[39] = 0xffffffff;
        expected.0[38] = 0xffffffff;
        expected.0[37] = 0xffffffff;
        expected.0[36] = 0xffffffff;
        expected.0[35] = 0xffffffff;
        expected.0[34] = 0xffffffff;
        expected.0[33] = 0xffffffff;
        expected.0[32] = 0xffffffff;
        expected.0[31] = 0xffffffff;
        expected.0[30] = 0xffffffff;
        expected.0[29] = 0xffffffff;
        expected.0[28] = 0xffffffff;
        expected.0[27] = 0xffffffff;
        expected.0[26] = 0xffffffff;
        expected.0[25] = 0xffffffff;
        expected.0[24] = 0xffffffff;
        expected.0[23] = 0xffffffff;
        expected.0[22] = 0xffffffff;
        expected.0[21] = 0xffffffff;
        expected.0[20] = 0xffffffff;
        expected.0[19] = 0xffffffff;
        expected.0[18] = 0xffffffff;
        expected.0[17] = 0xffffffff;
        expected.0[16] = 0xffffffff;
        expected.0[15] = 0xffffffff;
        expected.0[14] = 0xffffffff;
        expected.0[13] = 0xffffffff;
        expected.0[12] = 0x00303130;
        expected.0[11] = 0x0d060960;
        expected.0[10] = 0x86480165;
        expected.0[ 9] = 0x03040201;
        expected.0[ 8] = 0x05000420;
        expected.0[ 7] = 0x6daab02d;
        expected.0[ 6] = 0x35b47372;
        expected.0[ 5] = 0x03ba497d;
        expected.0[ 4] = 0x5cf4ce40;
        expected.0[ 3] = 0x115ce41c;
        expected.0[ 2] = 0x3ab4fd12;
        expected.0[ 1] = 0x61d4f575;
        expected.0[ 0] = 0x3503242d;
        let result = rsa_pubkey_op(&m, &n, &r, &rr, 65537, &mut work_4096o, &mut work_2048o, &mut work_2048, &mut work_2048_2);
        assert_eq!(result.0[..], expected.0[..]);
    }

    #[test]
    fn test_montmul() {
        let mut n = Bignum2048([0; 64]);
        let mut rr = Bignum2048([0; 64]);
        let mut a = Bignum2048([0; 64]);
        let mut b = Bignum2048([0; 64]);
        let mut expected = Bignum2048([0; 64]);

        let mut work_2048o = Bignum2048Oversized([0; 65]);
        let mut work_4096o = Bignum4096Oversized([0; 129]);

        n.0[63] = 0xd2fc5273;
        n.0[62] = 0xa2f9262b;
        n.0[61] = 0x84863bf6;
        n.0[60] = 0xe5b8fdd9;
        n.0[59] = 0x964e6190;
        n.0[58] = 0x2204cd46;
        n.0[57] = 0x46a16cac;
        n.0[56] = 0xca2d46cb;
        n.0[55] = 0x9a9ae44e;
        n.0[54] = 0xb1c387b6;
        n.0[53] = 0x395fee7e;
        n.0[52] = 0xa32cf051;
        n.0[51] = 0x8ff4bd02;
        n.0[50] = 0xe0b52ee9;
        n.0[49] = 0xa368fcc6;
        n.0[48] = 0xb8111d2b;
        n.0[47] = 0xaa1ad23b;
        n.0[46] = 0xfc741046;
        n.0[45] = 0x9701f53e;
        n.0[44] = 0x795d4599;
        n.0[43] = 0x01262d17;
        n.0[42] = 0x290c43a4;
        n.0[41] = 0x84faaef4;
        n.0[40] = 0xcfea8484;
        n.0[39] = 0x655231ec;
        n.0[38] = 0xd9bd5065;
        n.0[37] = 0xc35839ef;
        n.0[36] = 0x0b8dea96;
        n.0[35] = 0xe51bf71b;
        n.0[34] = 0xdcb821e8;
        n.0[33] = 0x57c3b561;
        n.0[32] = 0xf0ea71d9;
        n.0[31] = 0x57ac2755;
        n.0[30] = 0x4e1abc59;
        n.0[29] = 0x75897241;
        n.0[28] = 0x4c36c21b;
        n.0[27] = 0x6a402e4e;
        n.0[26] = 0x91b37aa9;
        n.0[25] = 0xf608ca52;
        n.0[24] = 0x9f60173d;
        n.0[23] = 0x8213f7fa;
        n.0[22] = 0x97ad73fa;
        n.0[21] = 0xab488831;
        n.0[20] = 0x757dc4b4;
        n.0[19] = 0x8f748958;
        n.0[18] = 0x2050589f;
        n.0[17] = 0xe265a540;
        n.0[16] = 0x526a7b3b;
        n.0[15] = 0xb2fa065c;
        n.0[14] = 0x0654f4b7;
        n.0[13] = 0x459bea90;
        n.0[12] = 0x544bae40;
        n.0[11] = 0x80da5f73;
        n.0[10] = 0x1e79b4d0;
        n.0[ 9] = 0x18426fdc;
        n.0[ 8] = 0xe7f15982;
        n.0[ 7] = 0x1ca1d21d;
        n.0[ 6] = 0xe3abf69b;
        n.0[ 5] = 0x738dd481;
        n.0[ 4] = 0x7f8e60de;
        n.0[ 3] = 0xe446e48c;
        n.0[ 2] = 0x8a986adb;
        n.0[ 1] = 0xfc4dbfe1;
        n.0[ 0] = 0xa3cec7a5;
        rr.0[63] = 0xa6a8b5f9;
        rr.0[62] = 0xfc683183;
        rr.0[61] = 0x660732ec;
        rr.0[60] = 0x7fad3138;
        rr.0[59] = 0x74d59bd3;
        rr.0[58] = 0x8cc3374a;
        rr.0[57] = 0x924f6690;
        rr.0[56] = 0xde3a3b5b;
        rr.0[55] = 0x8f8a7a64;
        rr.0[54] = 0x2c099f91;
        rr.0[53] = 0x33a45313;
        rr.0[52] = 0xf4a30ae3;
        rr.0[51] = 0x1fd6b74c;
        rr.0[50] = 0x1e5142af;
        rr.0[49] = 0x1feadbee;
        rr.0[48] = 0xa948a5b7;
        rr.0[47] = 0x7ffc2af3;
        rr.0[46] = 0x7c9bfd18;
        rr.0[45] = 0x6f00c434;
        rr.0[44] = 0xf489b5c9;
        rr.0[43] = 0x08850474;
        rr.0[42] = 0xc0e56809;
        rr.0[41] = 0x8ddef55d;
        rr.0[40] = 0xd0dc4f59;
        rr.0[39] = 0x08eb029e;
        rr.0[38] = 0x10f5315c;
        rr.0[37] = 0xae0ed8a3;
        rr.0[36] = 0x406dc80a;
        rr.0[35] = 0xe9406846;
        rr.0[34] = 0x6f10a46c;
        rr.0[33] = 0x9a741f79;
        rr.0[32] = 0x5d3feb16;
        rr.0[31] = 0x010023d9;
        rr.0[30] = 0x84de714b;
        rr.0[29] = 0x0943a775;
        rr.0[28] = 0x6f052d38;
        rr.0[27] = 0x5c36b89e;
        rr.0[26] = 0x6e6927c5;
        rr.0[25] = 0xcce5c6ff;
        rr.0[24] = 0x5aa1335a;
        rr.0[23] = 0x89b543e7;
        rr.0[22] = 0x02a9b47a;
        rr.0[21] = 0x00213bb7;
        rr.0[20] = 0x57469a06;
        rr.0[19] = 0xaeb9d46a;
        rr.0[18] = 0xdadf900a;
        rr.0[17] = 0xd209bb8a;
        rr.0[16] = 0x813c6fff;
        rr.0[15] = 0xb8e55d10;
        rr.0[14] = 0x7857c562;
        rr.0[13] = 0x78d195b1;
        rr.0[12] = 0xdabd2ea8;
        rr.0[11] = 0x6d72be73;
        rr.0[10] = 0xf01e2f77;
        rr.0[ 9] = 0x261925c3;
        rr.0[ 8] = 0xab87c57b;
        rr.0[ 7] = 0xe93885d2;
        rr.0[ 6] = 0x85400f34;
        rr.0[ 5] = 0xd83910ab;
        rr.0[ 4] = 0x687a0524;
        rr.0[ 3] = 0xcc9af25b;
        rr.0[ 2] = 0x2537f2f4;
        rr.0[ 1] = 0xc381e66e;
        rr.0[ 0] = 0x761158e0;
        a.0[0] = 123;
        b.0[0] = 456;
        expected.0[0] = 123 * 456;
        let a_mont = mont_mul(&a, &rr, &n, &mut work_4096o, &mut work_2048o);
        let b_mont = mont_mul(&b, &rr, &n, &mut work_4096o, &mut work_2048o);
        let result_mont = mont_mul(&a_mont, &b_mont, &n, &mut work_4096o, &mut work_2048o);
        let result = mont_red(&result_mont, &n, &mut work_4096o, &mut work_2048o);
        assert_eq!(result.0[..], expected.0[..]);
    }

    #[test]
    fn test_addhelper() {
        let mut x = [1, 2, 3];
        let cout = addhelper(&mut x, &[4, 5, 6]);
        assert_eq!(x, [5, 7, 9]);
        assert!(!cout);

        let mut x = [1, 2, 3];
        let cout = addhelper(&mut x, &[4, 5]);
        assert_eq!(x, [5, 7, 3]);
        assert!(!cout);

        let mut x = [4, 5, 6];
        let cout = addhelper(&mut x, &[1, 2]);
        assert_eq!(x, [5, 7, 6]);
        assert!(!cout);

        // Carry once
        let mut x = [3, 5, 6];
        let cout = addhelper(&mut x, &[0xFFFFFFFE, 2]);
        assert_eq!(x, [1, 8, 6]);
        assert!(!cout);

        let mut x = [3, 5, 6];
        let cout = addhelper(&mut x, &[2, 0xFFFFFFFE]);
        assert_eq!(x, [5, 3, 7]);
        assert!(!cout);

        // Carry twice
        let mut x = [3, 0xFFFFFFFE, 6];
        let cout = addhelper(&mut x, &[0xFFFFFFFE, 2]);
        assert_eq!(x, [1, 1, 7]);
        assert!(!cout);

        // Carry out
        let mut x = [3, 0xFFFFFFFE, 0xFFFFFFFF];
        let cout = addhelper(&mut x, &[0xFFFFFFFE, 2]);
        assert_eq!(x, [1, 1, 0]);
        assert!(cout);
    }

    #[test]
    fn test_mul2048() {
        let mut a = Bignum2048([0; 64]);
        let mut b = Bignum2048([0; 64]);
        let mut expected = Bignum4096Oversized([0; 129]);
        a.0[0] = 123;
        b.0[0] = 456;
        expected.0[0] = 123 * 456;
        let result = mul2048(&a, &b);
        assert_eq!(result.0[..], expected.0[..]);

        let mut a = Bignum2048([0; 64]);
        let mut b = Bignum2048([0; 64]);
        let mut expected = Bignum4096Oversized([0; 129]);
        a.0[0] = 123;
        b.0[0] = 456;
        b.0[1] = 789;
        expected.0[0] = 123 * 456;
        expected.0[1] = 123 * 789;
        let result = mul2048(&a, &b);
        assert_eq!(result.0[..], expected.0[..]);

        let mut a = Bignum2048([0; 64]);
        let mut b = Bignum2048([0; 64]);
        let mut expected = Bignum4096Oversized([0; 129]);
        a.0[31] = 0xf47b3353;
        a.0[30] = 0xb9e4b5dc;
        a.0[29] = 0x8d220eca;
        a.0[28] = 0xe5d1d5c8;
        a.0[27] = 0xaab639e9;
        a.0[26] = 0xd868b8b3;
        a.0[25] = 0x68c351dd;
        a.0[24] = 0x676a3327;
        a.0[23] = 0x81768a34;
        a.0[22] = 0x8146421e;
        a.0[21] = 0x284bd69d;
        a.0[20] = 0xcd5746ea;
        a.0[19] = 0xd78fefbe;
        a.0[18] = 0xaee85391;
        a.0[17] = 0x227363af;
        a.0[16] = 0xcaafa9ae;
        a.0[15] = 0x54cde700;
        a.0[14] = 0xb20526c2;
        a.0[13] = 0xb8c9dc7b;
        a.0[12] = 0x41e69ac1;
        a.0[11] = 0xd5618c4f;
        a.0[10] = 0x5e3bb9f6;
        a.0[ 9] = 0x174b7ed0;
        a.0[ 8] = 0x78f7ee71;
        a.0[ 7] = 0xaa30ea8d;
        a.0[ 6] = 0xd51025b0;
        a.0[ 5] = 0xb9ea8596;
        a.0[ 4] = 0x15f6c82d;
        a.0[ 3] = 0x80551ece;
        a.0[ 2] = 0xb4d62b5d;
        a.0[ 1] = 0x9b9fce43;
        a.0[ 0] = 0x4ebcdfe1;
        b.0[31] = 0xdced1d9e;
        b.0[30] = 0x55b6a74f;
        b.0[29] = 0xea14d9c8;
        b.0[28] = 0xda6c76f0;
        b.0[27] = 0x1de12b69;
        b.0[26] = 0x9db9e138;
        b.0[25] = 0x0cbad8f9;
        b.0[24] = 0xbaa327e5;
        b.0[23] = 0xdb667bb5;
        b.0[22] = 0xc7c23333;
        b.0[21] = 0x17639447;
        b.0[20] = 0xc1c56d03;
        b.0[19] = 0x8861dfd1;
        b.0[18] = 0x17b23e6a;
        b.0[17] = 0x480d8a51;
        b.0[16] = 0xe2e6e80c;
        b.0[15] = 0xbf221e26;
        b.0[14] = 0xde17284c;
        b.0[13] = 0x22c4cd49;
        b.0[12] = 0x2d53b717;
        b.0[11] = 0x28de8e05;
        b.0[10] = 0xf09cd610;
        b.0[ 9] = 0xfa0752ff;
        b.0[ 8] = 0x7ce6a82a;
        b.0[ 7] = 0xd25fddef;
        b.0[ 6] = 0xb57832ee;
        b.0[ 5] = 0xc6d2752b;
        b.0[ 4] = 0x22cd763b;
        b.0[ 3] = 0xdb57f82c;
        b.0[ 2] = 0x6eb6e8a4;
        b.0[ 1] = 0x8785d71c;
        b.0[ 0] = 0x37747045;
        expected.0[63] = 0xd2fc5273;
        expected.0[62] = 0xa2f9262b;
        expected.0[61] = 0x84863bf6;
        expected.0[60] = 0xe5b8fdd9;
        expected.0[59] = 0x964e6190;
        expected.0[58] = 0x2204cd46;
        expected.0[57] = 0x46a16cac;
        expected.0[56] = 0xca2d46cb;
        expected.0[55] = 0x9a9ae44e;
        expected.0[54] = 0xb1c387b6;
        expected.0[53] = 0x395fee7e;
        expected.0[52] = 0xa32cf051;
        expected.0[51] = 0x8ff4bd02;
        expected.0[50] = 0xe0b52ee9;
        expected.0[49] = 0xa368fcc6;
        expected.0[48] = 0xb8111d2b;
        expected.0[47] = 0xaa1ad23b;
        expected.0[46] = 0xfc741046;
        expected.0[45] = 0x9701f53e;
        expected.0[44] = 0x795d4599;
        expected.0[43] = 0x01262d17;
        expected.0[42] = 0x290c43a4;
        expected.0[41] = 0x84faaef4;
        expected.0[40] = 0xcfea8484;
        expected.0[39] = 0x655231ec;
        expected.0[38] = 0xd9bd5065;
        expected.0[37] = 0xc35839ef;
        expected.0[36] = 0x0b8dea96;
        expected.0[35] = 0xe51bf71b;
        expected.0[34] = 0xdcb821e8;
        expected.0[33] = 0x57c3b561;
        expected.0[32] = 0xf0ea71d9;
        expected.0[31] = 0x57ac2755;
        expected.0[30] = 0x4e1abc59;
        expected.0[29] = 0x75897241;
        expected.0[28] = 0x4c36c21b;
        expected.0[27] = 0x6a402e4e;
        expected.0[26] = 0x91b37aa9;
        expected.0[25] = 0xf608ca52;
        expected.0[24] = 0x9f60173d;
        expected.0[23] = 0x8213f7fa;
        expected.0[22] = 0x97ad73fa;
        expected.0[21] = 0xab488831;
        expected.0[20] = 0x757dc4b4;
        expected.0[19] = 0x8f748958;
        expected.0[18] = 0x2050589f;
        expected.0[17] = 0xe265a540;
        expected.0[16] = 0x526a7b3b;
        expected.0[15] = 0xb2fa065c;
        expected.0[14] = 0x0654f4b7;
        expected.0[13] = 0x459bea90;
        expected.0[12] = 0x544bae40;
        expected.0[11] = 0x80da5f73;
        expected.0[10] = 0x1e79b4d0;
        expected.0[ 9] = 0x18426fdc;
        expected.0[ 8] = 0xe7f15982;
        expected.0[ 7] = 0x1ca1d21d;
        expected.0[ 6] = 0xe3abf69b;
        expected.0[ 5] = 0x738dd481;
        expected.0[ 4] = 0x7f8e60de;
        expected.0[ 3] = 0xe446e48c;
        expected.0[ 2] = 0x8a986adb;
        expected.0[ 1] = 0xfc4dbfe1;
        expected.0[ 0] = 0xa3cec7a5;
        let result = mul2048(&a, &b);
        assert_eq!(result.0[..], expected.0[..]);
    }

    #[test]
    fn test_barrett() {
        let mut scratch_xr = [0u32; 25];
        let mut scratch2 = [0u32; 16];

        let mut x: [u32; 16] = [1234, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        barrett_reduction_core(&mut x, &secp256r1::BARRETT_MOD_N, &secp256r1::N,
            &mut scratch_xr, &mut scratch2);
        assert_eq!(x, [1234, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let mut x: [u32; 16] = [0xfc632551 + 1234, 0xf3b9cac2, 0xa7179e84, 0xbce6faad, 0xffffffff, 0xffffffff, 0x00000000, 0xffffffff,
        0, 0, 0, 0, 0, 0, 0, 0];
        barrett_reduction_core(&mut x, &secp256r1::BARRETT_MOD_N, &secp256r1::N,
            &mut scratch_xr, &mut scratch2);
        assert_eq!(x, [1234, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let mut x: [u32; 16] = [0xfc632551 + 1234, 0xf3b9cac2 + 5678, 0xa7179e84 + 666, 0xbce6faad, 0xffffffff, 0xffffffff, 0x00000000, 0xffffffff,
        0, 0, 0, 0, 0, 0, 0, 0];
        barrett_reduction_core(&mut x, &secp256r1::BARRETT_MOD_N, &secp256r1::N,
            &mut scratch_xr, &mut scratch2);
        assert_eq!(x, [1234, 5678, 666, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_inversemod() {
        let mut scratch0 = [0u32; 2];
        let mut scratch1 = [0u32; 2];
        let mut scratch2 = [0u32; 2];
        let mut scratch3 = [0u32; 2];

        let mut out = [0u32];

        inverse_mod_core(&[271], &[383], &mut out, &mut scratch0, &mut scratch1, &mut scratch2, &mut scratch3);
        assert_eq!(out, [106]);
    }

    #[test]
    fn test_pointdouble() {
        let result = point_double_projective(
            &ECPointProjective256{
                x: Bignum256(secp256r1::GxRmodP),
                y: Bignum256(secp256r1::GyRmodP),
                z: Bignum256(secp256r1::RmodP),
            });

        assert_eq!(result.x.0, [0x030756dd, 0x9396981f, 0x2a125983, 0x04e0f755, 0x3274ad78, 0x93bbc1aa, 0xd1eeeb17, 0x3216acbf]);
        assert_eq!(result.y.0, [0x3ccc950d, 0xdab96712, 0xaa13daa3, 0xcf48a95e, 0xce880027, 0xda98128d, 0x77bd1fde, 0xf9f84f1f]);
        assert_eq!(result.z.0, [0x90d6dc01, 0x1babad15, 0xa54a1d50, 0x8f93fb09, 0x1d7ebb0f, 0x6b812357, 0x3d1a282d, 0xc976e698]);
    }

    #[test]
    fn test_pointadd() {
        let result = point_add_projective_affine(
            &ECPointProjective256{
                x: Bignum256(secp256r1::GxRmodP),
                y: Bignum256(secp256r1::GyRmodP),
                z: Bignum256(secp256r1::RmodP),
            },
            &ECPointAffine256{
                x: Bignum256(secp256r1::GxRmodP),
                y: Bignum256(secp256r1::GyRmodP),
            });

        assert_eq!(result.x.0, [0x030756dd, 0x9396981f, 0x2a125983, 0x04e0f755, 0x3274ad78, 0x93bbc1aa, 0xd1eeeb17, 0x3216acbf]);
        assert_eq!(result.y.0, [0x3ccc950d, 0xdab96712, 0xaa13daa3, 0xcf48a95e, 0xce880027, 0xda98128d, 0x77bd1fde, 0xf9f84f1f]);
        assert_eq!(result.z.0, [0x90d6dc01, 0x1babad15, 0xa54a1d50, 0x8f93fb09, 0x1d7ebb0f, 0x6b812357, 0x3d1a282d, 0xc976e698]);

        let result = point_add_projective_affine(
            &result,
            &ECPointAffine256{
                x: Bignum256(secp256r1::GxRmodP),
                y: Bignum256(secp256r1::GyRmodP),
            });

        assert_eq!(result.x.0, [0x950ca643, 0x451e0d5f, 0xd94ab1ba, 0x7fe9955c, 0xad684b8a, 0xc0e844b0, 0xc4f66773, 0x2dcb9402]);
        assert_eq!(result.y.0, [0xb44fa0bb, 0xa82aff58, 0xd3be6cee, 0xad252196, 0x68972739, 0xe9e0aab0, 0x88dba91f, 0xf89113fc]);
        assert_eq!(result.z.0, [0x50cdca81, 0xa0e7b097, 0x0ae3b2ac, 0xba93bdaa, 0x210fd88a, 0xb6d954ca, 0x4b520bc0, 0x0f8bea77]);
    }

    #[test]
    fn test_shamir_trick() {
        let pubk = ECPointAffine256 {
            x: Bignum256([0x4f33f883, 0xffdc3ff2, 0x836af311, 0x38b3c0c0, 0xe34e8a8d, 0x919b1318, 0x57bc232f, 0x91c89152]),
            y: Bignum256([0x375c37c9, 0x8e1e4703, 0xf91b2e47, 0x09788b02, 0x724d2c63, 0x23f98d7a, 0xbc3c93e6, 0x94cee68f]),
        };

        let pubk_plus_g = ECPointAffine256 {
            x: Bignum256([0xffa047e9, 0x8a97f91b, 0x9904be31, 0x3cc60dfb, 0x036b51ff, 0x94b25916, 0xf796d136, 0x19e17ade]),
            y: Bignum256([0x086d4036, 0x89cdf37b, 0x66f59ea6, 0x8eb771b3, 0x008da49c, 0x8a63633a, 0xc8705153, 0x7487e759]),
        };

        let k1 = Bignum256([123, 0, 0, 0, 0, 0, 0, 0]);
        let k2 = Bignum256([456, 0, 0, 0, 0, 0, 0, 0]);

        let result = ec_two_scalar_mult_shamir(&k1, &k2, &pubk, &pubk_plus_g);

        assert_eq!(result.x.0, [0xaa784b77, 0x9e13406c, 0x618f4728, 0xc4c955a3, 0xd81442c7, 0x2cefb714, 0xe3e7a23d, 0x9cde435d]);
        assert_eq!(result.y.0, [0x02b674eb, 0x492bb57c, 0x83bb9494, 0x09f4f4ff, 0x055cf766, 0xda69ef7d, 0xfe60d1bb, 0xde94ef02]);
        assert_eq!(result.z.0, [0x8659f4a9, 0xb9dc8532, 0x7b0423b7, 0x4e3f9445, 0xa9ecc47b, 0x8823677b, 0x6077dbbb, 0x33c923c8]);
    }

    #[test]
    fn test_ecdsa() {
        let pubk = ECPointAffine256 {
            x: Bignum256([0x4f33f883, 0xffdc3ff2, 0x836af311, 0x38b3c0c0, 0xe34e8a8d, 0x919b1318, 0x57bc232f, 0x91c89152]),
            y: Bignum256([0x375c37c9, 0x8e1e4703, 0xf91b2e47, 0x09788b02, 0x724d2c63, 0x23f98d7a, 0xbc3c93e6, 0x94cee68f]),
        };

        let pubk_plus_g = ECPointAffine256 {
            x: Bignum256([0xffa047e9, 0x8a97f91b, 0x9904be31, 0x3cc60dfb, 0x036b51ff, 0x94b25916, 0xf796d136, 0x19e17ade]),
            y: Bignum256([0x086d4036, 0x89cdf37b, 0x66f59ea6, 0x8eb771b3, 0x008da49c, 0x8a63633a, 0xc8705153, 0x7487e759]),
        };

        let hash = Bignum256([0xa0674d33, 0xfd873996, 0x6eb811f9, 0x124cf236, 0xd0f1f9d1, 0x01239f7e, 0xb86cfa0e, 0x403dee8c]);
        let sig_r = Bignum256([0xe99aac39, 0x7c2b237b, 0xb0024f02, 0x19e7c3f9, 0x97fec477, 0xdd847472, 0xd2a547d0, 0x811a6c2b]);
        let sig_s = Bignum256([0xb5a1d63c, 0x7393a694, 0x9daa5595, 0xcb2ba405, 0xf7f668a7, 0xb41a7db0, 0xf29c820c, 0xb428639a]);

        assert!(ecdsa_secp256r1_verify_sig(&pubk, &pubk_plus_g, &hash, &sig_r, &sig_s));
    }

    #[test]
    #[ignore]
    fn test_modinv32() {
        for x in 0..2147483647 {
            let x = 2 * x + 1;  // Odd numbers only
            let res = modinv32(x);
            assert_eq!(x.wrapping_mul(res), 1);
        }
    }
}

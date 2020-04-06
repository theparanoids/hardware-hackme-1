// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use ::hwrng_get_u32;

macro_rules! chacha20_quarterround {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {{
        $a = $a.wrapping_add($b);   $d = ($d ^ $a).rotate_left(16);
        $c = $c.wrapping_add($d);   $b = ($b ^ $c).rotate_left(12);
        $a = $a.wrapping_add($b);   $d = ($d ^ $a).rotate_left(8);
        $c = $c.wrapping_add($d);   $b = ($b ^ $c).rotate_left(7);
    }}
}

pub fn chacha20_raw(inp: &[u32; 16]) -> [u32; 16] {
    // Copy the input
    let mut x = *inp;
    // Scramble the copy
    for _ in 0..10 {
        chacha20_quarterround!(x[0], x[4],  x[8], x[12]);
        chacha20_quarterround!(x[1], x[5],  x[9], x[13]);
        chacha20_quarterround!(x[2], x[6], x[10], x[14]);
        chacha20_quarterround!(x[3], x[7], x[11], x[15]);

        chacha20_quarterround!(x[0], x[5], x[10], x[15]);
        chacha20_quarterround!(x[1], x[6], x[11], x[12]);
        chacha20_quarterround!(x[2], x[7],  x[8], x[13]);
        chacha20_quarterround!(x[3], x[4],  x[9], x[14]);
    }
    // Return scrambled data plus the original
    for i in 0..16 {
        x[i] = x[i].wrapping_add(inp[i]);
    }
    x
}

#[repr(C)]
pub struct ChaCha20PRNGState {
    pub secretbuf: [u32; 16],
    pub cached_output: [u32; 16],
    pub cache_byte_idx: usize,
}

impl ChaCha20PRNGState {
    pub fn new(rng: &::stm32f405::RNG, last_rng_val: &mut u32) -> Self {
        let phase1 = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, // "expand 32-byte k"
                        // key
                        hwrng_get_u32(rng, last_rng_val), hwrng_get_u32(rng, last_rng_val),
                        hwrng_get_u32(rng, last_rng_val), hwrng_get_u32(rng, last_rng_val),
                        hwrng_get_u32(rng, last_rng_val), hwrng_get_u32(rng, last_rng_val),
                        hwrng_get_u32(rng, last_rng_val), hwrng_get_u32(rng, last_rng_val),
                        // counter
                        0, 0,
                        // nonce
                        hwrng_get_u32(rng, last_rng_val), hwrng_get_u32(rng, last_rng_val)];

        let phase1_output = chacha20_raw(&phase1);

        let phase2 = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, // "expand 32-byte k"
                        // key
                        phase1_output[0], phase1_output[1], phase1_output[2], phase1_output[3],
                        phase1_output[4], phase1_output[5], phase1_output[6], phase1_output[7],
                        // counter
                        0, 0,
                        // nonce
                        phase1_output[8], phase1_output[9]];

        let phase2_output = chacha20_raw(&phase2);

        Self {
            secretbuf: phase2,
            cached_output: phase2_output,
            cache_byte_idx: 0,
        }
    }

    pub fn broken(seed: &[u32; 10]) -> Self {
        let phase2 = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, // "expand 32-byte k"
                        // key
                        seed[0], seed[1], seed[2], seed[3],
                        seed[4], seed[5], seed[6], seed[7],
                        // counter
                        0, 0,
                        // nonce
                        seed[8], seed[9]];

        let phase2_output = chacha20_raw(&phase2);

        Self {
            secretbuf: phase2,
            cached_output: phase2_output,
            cache_byte_idx: 0,
        }
    }

    pub fn getu8(&mut self) -> u8 {
        if self.cache_byte_idx == self.cached_output.len() * 4 {
            // Need more data
            self.cache_byte_idx = 0;

            // Increment counter
            self.secretbuf[12] = self.secretbuf[12].wrapping_add(1);
            if self.secretbuf[12] == 0 {
                self.secretbuf[13] = self.secretbuf[13].wrapping_add(1);
            }

            self.cached_output = chacha20_raw(&self.secretbuf);
        }

        let word_idx = self.cache_byte_idx / 4;
        let byte_idx = self.cache_byte_idx % 4;
        let word = self.cached_output[word_idx];
        let byte = (word >> (byte_idx * 8)) as u8;
        self.cache_byte_idx += 1;

        byte
    }
}

// This "crapto" is for level 5
pub struct LCGPRNGState {
    pub state: u32,
}

impl LCGPRNGState {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed & 0x7FFFFFFF
        }
    }

    pub fn getu32(&mut self) -> u32 {
        let new_val = self.state.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFFFFFF;
        self.state = new_val;
        new_val
    }

    pub fn getu8(&mut self) -> u8 {
        let x = self.getu32();
        (((x >> 24) & 0xFF) ^ ((x >> 16) & 0xFF) ^ ((x >> 8) & 0xFF) ^ (x & 0xFF)) as u8
    }
}

// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::mem;
use core::ptr;

/// Write a u32 into a vector, which must be 4 bytes long. The value is written in little-endian
/// format.
pub fn write_u32_le(dst: &mut[u8], mut input: u32) {
    assert!(dst.len() == 4);
    input = input.to_le();
    unsafe {
        let tmp = &input as *const _ as *const u8;
        ptr::copy_nonoverlapping(tmp, dst.get_unchecked_mut(0), 4);
    }
}

/// Read a vector of bytes into a vector of u32s. The values are read in little-endian format.
pub fn read_u32v_le(dst: &mut[u32], input: &[u8]) {
    assert!(dst.len() * 4 == input.len());
    unsafe {
        let mut x: *mut u32 = dst.get_unchecked_mut(0);
        let mut y: *const u8 = input.get_unchecked(0);
        for _ in 0..dst.len() {
            let mut tmp: u32 = mem::uninitialized();
            ptr::copy_nonoverlapping(y, &mut tmp as *mut _ as *mut u8, 4);
            *x = u32::from_le(tmp);
            x = x.offset(1);
            y = y.offset(4);
        }
    }
}

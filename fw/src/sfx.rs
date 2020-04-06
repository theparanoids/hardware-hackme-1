// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

#![allow(dead_code)]

extern crate stm32f405;

const HALF: u32 = 1200;
const QUARTER: u32 = 600;
const EIGHT: u32 = 300;
const SIXTEENTH: u32 = 150;

const C4: f32   = 261.63;
const C4S: f32  = 277.18;
const D4: f32   = 293.66;
const D4S: f32  = 311.13;
const E4: f32   = 329.63;
const F4: f32   = 349.23;
const F4S: f32  = 369.99;
const G4: f32   = 392.00;
const G4S: f32  = 415.30;
const A4: f32   = 440.00;
const A4S: f32  = 466.16;
const B4: f32   = 493.88;
const C5: f32   = 523.25;
const C5S: f32  = 554.37;
const D5: f32   = 587.33;
const D5S: f32  = 622.25;
const E5: f32   = 659.25;
const F5: f32   = 698.46;
const F5S: f32  = 739.99;
const G5: f32   = 783.99;
const G5S: f32  = 830.61;
const A5: f32   = 880.00;
const A5S: f32  = 932.33;
const B5: f32   = 987.77;
const C6: f32   = 1046.50;

pub const FOUR_NOTE_COMPLETE_SFX: &[(f32, u32)] = &[
    (A4, EIGHT),
    (A4S, EIGHT),
    (B4, EIGHT),
    (C5, 750)
];

pub fn speaker_set_freq(tim2: &stm32f405::TIM2, freq: f32) {
    // Still not sure why 168 MHz
    let divider_i = if freq != 0. {
        let divider_f = 2. * (::APB1FREQ as f32) / freq;
        divider_f as u32
    } else {
        0
    };

    unsafe {
        tim2.arr.write(|w| w.bits(divider_i));
        tim2.ccr4.write(|w| w.bits(divider_i / 2));
    }
    tim2.egr.write(|w| w.ug().bit(true));       // Kick UG
}

pub fn play_song(r: &::idle::Resources, song: &[(f32, u32)]) {
    for &(note, time) in song {
        speaker_set_freq(r.SPEAKER_TIMER, note);
        ::msleep(r.IDLE_MS_COUNTER, time);
    }
    speaker_set_freq(r.SPEAKER_TIMER, 0.);
}

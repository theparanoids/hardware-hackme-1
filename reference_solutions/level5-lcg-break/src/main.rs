// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use std::io::{self, BufRead};

extern crate rayon;
use rayon::prelude::*;

/// The following two functions are a reimplementation of the vulnerable RNG logic in the level 5
/// challenge. In a CTF scenario, the attacker would have to either guess this or be given this
/// information in a hint.

// Given a particular seed value (a 31-bit number), turn it into a dice roll (1-6). If the seed
// happens to fall into the "need to re-roll" range, returns None instead.
fn next_dice_roll_at_seed(x: u32) -> Option<u8> {
    let xx = ((x >> 24) & 0xFF) ^ ((x >> 16) & 0xFF) ^ ((x >> 8) & 0xFF) ^ (x & 0xFF);
    if xx < 252 {
        let guess_num = xx % 6;
        Some((guess_num + 1) as u8)
    } else {
        None
    }
}

// Given a particular seed value, compute the next seed value after rolling the dice once.
fn next_seed(mut x: u32) -> u32 {
    loop {
        // This following line of code implements the LCG PRNG logic (the logic used in rand()).
        x = x.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFFFFFF;
        // This logic (including the loop) handles the "need to re-roll" logic.
        let xx = ((x >> 24) & 0xFF) ^ ((x >> 16) & 0xFF) ^ ((x >> 8) & 0xFF) ^ (x & 0xFF);
        if xx < 252 {
            return x;
        }
    }
}

/// Implements the actual PRNG breaking logic.

fn main() {
    // Big array containing all possible seed values that are currently possible
    let mut seeds_possible = Vec::with_capacity(0x80000000);
    for i in 0..0x80000000u32 {
        seeds_possible.push(i);
    }

    loop {
        println!("There are currently {} possible seeds.", seeds_possible.len());

        if seeds_possible.len() == 0 {
            panic!("should not have zero seeds left");
        }

        // If there is only one seed left, we are done. We can go on to predicting future dice rolls.
        if seeds_possible.len() == 1 {
            let mut found_seed = seeds_possible[0];

            loop {
                // Predict a dice roll
                let dice_roll = next_dice_roll_at_seed(found_seed).unwrap();
                let seed_next = next_seed(found_seed);

                println!("Next value is {} (seed {:08X})", dice_roll, seed_next);
                // Wait for the user to press enter before continuing.
                let mut inp = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut inp).unwrap();

                found_seed = seed_next;
            }
        }

        println!("What is the next dice roll?");
        // Read the input from the user and convert it to an integer
        let mut inp = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut inp).unwrap();
        let actual_num = inp.trim().parse::<u8>().unwrap();

        // Use Rayon for automagic multithreading
        seeds_possible = seeds_possible.into_par_iter()
            // Eliminate all seeds that don't work
            .filter(|&seed| {
                // Compute what dice roll the current seed value would result in
                let this_seed_dice_roll = next_dice_roll_at_seed(seed);
                // The seed remains possible if it doesn't require a re-roll and matches the number
                // input by the user
                this_seed_dice_roll.is_some() && this_seed_dice_roll.unwrap() == actual_num
            })
            // Compute the next seed value for all of the current seeds that are still possible
            .map(|seed| {
                next_seed(seed)
            })
            .collect();
    }
}

// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

use std::process::Command;

fn main() {
    // git revision
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let git_hash = git_hash.trim();
    // is working directory clean?
    let output = Command::new("git").args(&["status", "--porcelain", "-uno"]).output().unwrap();
    let git_dirty = if output.stdout.len() != 0 {"-dirty"} else {""};
    println!("cargo:rustc-env=GIT_HASH={}{}", git_hash, git_dirty);
}

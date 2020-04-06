[![crates.io](https://img.shields.io/crates/v/cortex-m-rt.svg)](https://crates.io/crates/cortex-m-rt)
[![crates.io](https://img.shields.io/crates/d/cortex-m-rt.svg)](https://crates.io/crates/cortex-m-rt)

# Hacked `cortex-m-rt`

This is a hacked version of `cortex-m-rt` where the `main` function has been
replaced by a function called `hijack_main`. This was needed in order to add
code to enable certain hardware peripherals and manipulate the stack pointer
before the RTFM framework's main function is called. The original readme
follows.

# `cortex-m-rt`

> Minimal runtime / startup for Cortex-M microcontrollers

# [Documentation](https://docs.rs/cortex-m-rt)

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

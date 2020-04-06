# Paranoids Hardware Hackme 1
> Firmware for a hackme designed by the Paranoids security team.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Configuration](#configuration)
- [Usage](#usage)
- [Security](#security)
- [Contribute](#contribute)
- [License](#license)

## Background

This firmware runs on [a PCB](https://github.com/yahoo/hardware-hackme-1-eda) designed by the Paranoids security team. The firmware implements a capture-the-flag challenge focused around embedded systems security. In a capture-the-flag challenge, a system is designed with deliberate security weaknesses, and the goal for the person playing the challenge is to find and exploit these weaknesses. This firmware is built in Rust and runs on an STM32F4 microcontroller.

## Install

To compile the main component of the firmware, change into the `fw/` directory and run `xargo build --release`. The resulting firmware will be in `target/thumbv7em-none-eabihf/release/paranoids-hackme-1-fw`. To flash the firmware to a board, load this file into arm-none-eabi-gdb and run commands similar to the following:

```
$ arm-none-eabi-gdb target/thumbv7em-none-eabihf/release/paranoids-hackme-1-fw
(gdb) target extended-remote /dev/cu.usbmodemXXXXXXXX
(gdb) monitor swdp_scan
(gdb) attach 1
(gdb) load
```

Once the firmware is flashed, `fw/provisioner.py` is used to generate private keys and otherwise set up the target board so that it is ready to be used.

By default, the provisioning process will enable level 1 flash readback protection on the STM32 microcontroller. This is necessary because some of the levels store "answers" in the flash memory. If readback protection were not enabled, an attacker could directly extract these answers using the debugger. However, enabling flash readback protection will interfere with attempting to load new firmware. In order to load new firmware onto a board with readback protection enabled, the firmware contains a secret undocumented command `__unlockme_this_will_brick_the_device` that will unlock the flash with a side effect of erasing the entire firmware. This command is entered at the firmware main menu. After entering the command, **wait several seconds** until the LEDs near the microcontroller blink. At this point, the board has to be fully reset (unplugging and replugging the USB cable), after which it should be unlocked. If for some reason this command was not successful, the flash readback protection can also be reset using GDB:

```
(gdb) set lang c
(gdb) set {int}0x40023c08 = 0x08192A3B
(gdb) set {int}0x40023c08 = 0x4C5D6E7F
(gdb) set {int}0x40023c14 = 0x0FFFAAEC
(gdb) set {int}0x40023c14 = 0x0FFFAAEE
```

To recompile the CPLD firmware, consult the separate README.md in the `cpld/` directory.

## Configuration

This firmware is written in the Rust programming language and requires a Rust compiler to build. The recommended way to install Rust is to use the [rustup](https://rustup.rs/) installer. The firmware requires exactly the nightly from 2018-04-07. This can be installed using the `rustup toolchain install nightly-2018-04-07` command. The [GNU Arm Embedded Toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm) is also required in order to obtain ld and gdb. These tools should be added to `PATH`. The following commands can be used to verify that the tools are installed correctly:

```
$ rustc --version
rustc 1.27.0-nightly (eeea94c11 2018-04-06)
$ arm-none-eabi-ld --version
GNU ld (GNU Tools for Arm Embedded Processors 7-2017-q4-major) 2.29.51.20171128
Copyright (C) 2017 Free Software Foundation, Inc.
This program is free software; you may redistribute it under the terms of
the GNU General Public License version 3 or (at your option) a later version.
This program has absolutely no warranty.
```

Finally, xargo is required and can be installed with `cargo install xargo`.

For additional guidance, follow the historical embedded Rust quickstart [here](https://blog.japaric.io/quickstart/).

The provisioning tool and other miscellaneous Python scripts require dependencies listed in `fw/requirements.txt`. These can be installed using `pip`.

To recompile the CPLD firmware, Xilinx ISE is required. Consult the separate README.md in the `cpld/` directory.

## Usage

Once the firmware is flashed and keys are loaded, the challenge can be played by attaching a terminal emulator program to the second USB serial port. This console runs at 115200 baud and should generate a `> ` prompt. `help` will give basic usage instructions.

## Security

This project contains custom code implementing cryptographic primitives (RSA and ECDSA). Although these algorithms have been tested and appear to generally work correctly, it has not been vetted as thoroughly as other cryptographic libraries. In addition, signing functions in this code do not implement any kind of side channel protection. This code is primarily intended for demonstration purposes only.

## Contribute

Please refer to [the contributing.md file](Contributing.md) for information about how to get involved. We welcome issues, questions, and pull requests.

## Maintainers
- R. Ou: robert.ou@verizonmedia.com

## License
This project is licensed under the terms of the [MIT](LICENSE-MIT) open source license. Please refer to [LICENSE](LICENSE) for the full terms.

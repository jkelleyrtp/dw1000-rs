# DWM1001 Board Support Crate [![crates.io](https://img.shields.io/crates/v/dwm1001.svg)](https://crates.io/crates/dwm1001) [![Documentation](https://docs.rs/dwm1001/badge.svg)](https://docs.rs/dwm1001) [![Build Status](https://travis-ci.org/braun-embedded/rust-dwm1001.svg?branch=master)](https://travis-ci.org/braun-embedded/rust-dwm1001)

## Introduction

Board support crate for the Decawave [DWM1001 module] and [DWM1001 development board] board, written in the [Rust] programming language.

[DWM1001 module]: https://www.decawave.com/product/dwm1001-module/
[DWM1001 development board]: https://www.decawave.com/product/dwm1001-development-board/
[Rust]: https://www.rust-lang.org/


## Status

This crate itself is relatively stable and complete, but is still missing some features. Be mindful that its API exposes a number of other crates, whose development status varies.

This project is still in development. No guarantee of API stability is made, so expect future versions to require updates in your code.


## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies.dwm1001]
version = "0.2"
```

This crate exposes various Cargo features that are useful in various situations, none of which is enabled by default:

- `dev`: Exposes the features of the DWM1001 development board. If you're working with the DWM1001 development board, as opposed to a bare DWM1001 module, enable this feature.
- `rt`: Enables runtime features. This is required if you're writing an application. Libraries should not enable this feature.
- `semihosting`: Enable debug output via semihosting. Enable this feature only if you need it. If you enable this feature without being connected to a host, the program on the microcontroller won't run.

To build, upload and run an applicatio built on this library, you need working configuration for Cargo, [cortex-m-rt], [OpenOCD] and GDB. You can use `.cargo/config`, `openocd.cfg`, `memory.x`, and `.gdbinit` from this repository as a starting point.

[cortex-m-rt]: https://crates.io/crates/cortex-m-rt
[OpenOCD]: http://openocd.org/


## Documentation

Please refer to the **[API Reference]** for further documentation.

[Example programs] are available in the GitHub repository.

[API Reference]: https://docs.rs/dwm1001
[Example programs]: https://github.com/braun-embedded/rust-dwm1001/tree/master/examples


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-embedded/rust-dwm1001/blob/master/LICENSE


**Created by [Braun Embedded](https://braun-embedded.com/)** <br />
**Initial development sponsored by [Ferrous Systems](https://ferrous-systems.com/)**

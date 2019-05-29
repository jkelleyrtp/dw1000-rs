# Rust DW1000 Driver [![crates.io](https://img.shields.io/crates/v/dw1000.svg)](https://crates.io/crates/dw1000) [![Documentation](https://docs.rs/dw1000/badge.svg)](https://docs.rs/dw1000) [![Build Status](https://travis-ci.org/braun-embedded/rust-dw1000.svg?branch=master)](https://travis-ci.org/braun-embedded/rust-dw1000)

## Introduction

Driver for the Decawave [DW1000] UWB transceiver, written in the [Rust] programming language. If you're using the DW1000 with a DWM1001 module or a DWM1001-Dev board, please check out the [DWM1001 crate].

[DW1000]: https://www.decawave.com/products/dw1000
[Rust]: https://www.rust-lang.org/
[DWM1001 crate]: https://crates.io/crates/dwm1001


## Status

This driver covers the main features of the DW1000, wireless communication and distance measurement, although the distance measurement is currently lacking range bias compensation, making it somewhat imprecise.

As of this writing, the driver is well-tested ([examples] are available in the rust-dwm1001 repository), but has yet to be proven in real-world use cases.

This project is still in development. No guarantee of API stability is made, so expect future versions to require updates in your code.

[examples]: https://github.com/braun-robotics/rust-dwm1001/tree/master/examples


## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw1000 = "0.2"
```


## Documentation

Please refer to the **[API Reference]**.

[Example programs] are available in the rust-dwm1001 repository.

[API Reference]: https://docs.rs/dw1000
[Example programs]: https://github.com/braun-robotics/rust-dwm1001/tree/master/examples


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-robotics/rust-dw1000/blob/master/LICENSE


**Created by [Braun Robotics](https://braun-robotics.com/)** <br />
**Initial development sponsored by [Ferrous Systems](https://ferrous-systems.com/)**

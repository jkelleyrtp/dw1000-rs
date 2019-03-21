# Rust DW1000 Driver [![crates.io](https://img.shields.io/crates/v/dw1000.svg)](https://crates.io/crates/dw1000) [![Documentation](https://docs.rs/dw1000/badge.svg)](https://docs.rs/dw1000) [![Build Status](https://travis-ci.org/braun-robotics/rust-dw1000.svg?branch=master)](https://travis-ci.org/braun-robotics/rust-dw1000)

## Introduction

Driver for the Decawave [DW1000] UWB transceiver, written in the [Rust] programming language. If you're using the DW1000 with a DWM1001 module or a DWM1001-Dev board, please check out the [DWM1001 crate].

[DW1000]: https://www.decawave.com/products/dw1000
[Rust]: https://www.rust-lang.org/
[DWM1001 crate]: https://crates.io/crates/dwm1001


## Status

This driver covers the main features of the DW1000, wireless communication and distance measurement, although the distance measurement is currently lacking range bias compensation, making it somewhat imprecise.

As of this writing, the driver is well-tested (see [examples]), but has yet to be proven in real-world use cases.

This project is still in development. No guarantee of API stability is made, so expect future versions to require updates in your code.


## Usage

Include this crate in your Cargo project by adding the following to `Cargo.toml`:
```toml
[dependencies]
dw1000 = "0.1"
```

You can run the examples in this repository on a [DWM1001 Development Board]. If you have [OpenOCD] and [arm-none-eabi-gdb] installed, and the DWM1001 dev board connected via USB, you should be able to connect to the board using OpenOCD:

```
$ openocd
```

Then you should be able to run any example like this:

```
$ cargo run --release --example reg_rw
```

To enable debug output run the same command, but with the `semihosting` feature enabled:

```
$ cargo run --release --example reg_rw --features=semihosting
```

The output will be printed by OpenOCD.

Please note that examples that are compiled with semihosting enabled won't run without a connection to OpenOCD. If you want to flash an example that can then run independently of your host computer, make sure to forgo the `semihosting` feature.

[DWM1001 Development Board]: https://www.decawave.com/product/dwm1001-development-board/
[OpenOCD]: http://openocd.org/
[arm-none-eabi-gdb]: https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads


## Documentation

Please refer to the **[API Reference]** and the [examples].

[Example programs] are available in the rust-dwm1001 repository.

[API Reference]: https://docs.rs/dw1000


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-robotics/rust-dw1000/blob/master/LICENSE


**Created by [Braun Robotics](https://braun-robotics.com/)** <br />
**Initial development sponsored by [Ferrous Systems](https://ferrous-systems.com/)**


[examples]: https://github.com/braun-robotics/rust-dw1000/tree/master/examples

# DWM1001

## Introduction

Board support crate for the Decawave [DWM1001]/[DWM1001-Dev] board, written in the [Rust] programming language. This crate is in early development. Not much to see here right now.

[DWM1001]: http://www.decawave.com/products/dwm1001-module
[DWM1001-Dev]: https://www.decawave.com/products/dwm1001-dev
[Rust]: https://www.rust-lang.org/


## Usage

To run the example, execute the following command in the project root:

```
cargo run --example blink --features rt
```

If a DWM1001-Dev board is connected to your computer via USB, this should upload and run the example on that board. This requires [OpenOCD] and the `arm-none-eabi` GCC toolchain to be installed.

To use this crate in your library, add the following to your `Cargo.toml`:

``` toml
[dependencies]
dwm1001 = { git = "https://github.com/braun-robotics/rust-dwm1001.git" }
```

To use this crate in your application, you also need to enable the `rt` feature. Add the following to your `Cargo.toml`:

``` toml
[dependencies.dwm1001]
git      = "https://github.com/braun-robotics/rust-dwm1001.git"
features = "rt"
```

To build, upload and run your application, you need working configuration for Cargo, [cortex-m-rt], OpenOCD and GDB. You can use `.cargo/config`, `openocd.cfg`, `memory.x`, and `.gdbinit` from this repository as a starting point.

[OpenOCD]: http://openocd.org/
[cortex-m-rt]: https://crates.io/crates/cortex-m-rt


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-robotics/rust-dwm1001/blob/master/LICENSE


**Supported by [Braun Robotics](https://braun-robotics.com/)**

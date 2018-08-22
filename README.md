# DW1000

## Introduction

Driver for the Decawave [DW1000] UWB transceiver, written in the [Rust] programming language. If you're using the DW1000 with a DWM1001 module or'a DWM1001-Dev board, please check out the [DWM1001 crate].

[DW1000]: https://www.decawave.com/products/dw1000
[Rust]: https://www.rust-lang.org/
[DWM1001 crate]: https://github.com/braun-robotics/rust-dwm1001


## Status

This crate is in very early development. The crate currently depends on [nrf52-hal], the HAL API for the nRF52, which is used on the DWM1001 module. This is fine if you're using the DW1000 with the DWM1001, but unfortunately it means you can't use this crate with any other microcontroller right now.

This is a temporary state, until the [embedded-hal] support of nrf52-hal improves to the point where this crate can depend on embedded-hal instead.

[nrf52-hal]: https://github.com/jamesmunns/nrf52-hal
[embedded-hal]: https://github.com/rust-embedded/embedded-hal


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-robotics/rust-dwm1001/blob/master/LICENSE


**Supported by [Braun Robotics](https://braun-robotics.com/)**

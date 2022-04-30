//! High-level interface to the DW1000
//!
//! The entry point to this API is the [DW1000] struct. Please refer to the
//! documentation there for more details.
//!
//! This module implements a high-level interface to the DW1000. This is the
//! recommended way to access the DW1000 using this crate, unless you need the
//! greater flexibility provided by the [register-level interface].
//!
//! [register-level interface]: ../ll/index.html

use crate::{ll, time::Duration, RxConfig, TxConfig};
use core::{fmt, num::Wrapping};

pub use awake::*;
pub use error::*;
pub use ready::*;
pub use receiving::*;
pub use sending::*;
pub use sleeping::*;
pub use state_impls::*;
pub use uninitialized::*;

mod awake;
mod error;
mod ready;
mod receiving;
mod sending;
mod sleeping;
mod state_impls;
mod uninitialized;

/// Entry point to the DW1000 driver API
pub struct DW1000<SPI, CS> {
    ll: ll::DW1000<SPI, CS>,
    seq: Wrapping<u8>,
    state: DW1000Status,
    pub(crate) tx_cfg: TxConfig,
    pub(crate) rx_cfg: RxConfig,
}

/// The current status of the DWM module
#[derive(Debug)]
pub(crate) enum DW1000Status {
    Ready,
    Sleeping { tx_antenna_delay: Duration },
    Sending,
    AutoDoubleBufferReceiving,
    SingleBufferReceiving,
}

// Can't be derived without putting requirements on `SPI` and `CS`.
impl<SPI, CS> fmt::Debug for DW1000<SPI, CS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DW1000 {{ state: ")?;
        self.state.fmt(f)?;
        write!(f, ", .. }}")?;

        Ok(())
    }
}

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

use crate::ll;
use core::{fmt, num::Wrapping};

pub use error::*;
pub use ready::*;
pub use receiving::*;
pub use state_impls::*;

mod awake;
mod error;
mod ready;
mod receiving;
mod sending;
mod sleeping;
mod state_impls;
mod uninitialized;

/// Entry point to the DW1000 driver API
pub struct DW1000<SPI, State> {
    ll: ll::DW1000<SPI>,
    seq: Wrapping<u8>,
    state: State,
}

// Can't be derived without putting requirements on `SPI` and `CS`.
impl<SPI, State> fmt::Debug for DW1000<SPI, State>
where
    State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DW1000 {{ state: ")?;
        self.state.fmt(f)?;
        write!(f, ", .. }}")?;

        Ok(())
    }
}

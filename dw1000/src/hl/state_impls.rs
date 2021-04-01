use crate::{time::Duration, RxConfig};

/// Indicates that the `DW1000` instance is not initialized yet
#[derive(Debug)]
pub struct Uninitialized;

/// Indicates that the `DW1000` instance is ready to be used
#[derive(Debug)]
pub struct Ready;

/// Indicates that the `DW1000` instance is currently sending
#[derive(Debug)]
pub struct Sending {
    pub(super) finished: bool,
}

/// Indicates that the `DW1000` instance is currently receiving
#[derive(Debug)]
pub struct Receiving {
    pub(super) finished: bool,
    pub(super) used_config: RxConfig,
}

/// Indicates that the `DW1000` instance is currently sleeping
#[derive(Debug)]
pub struct Sleeping {
    /// Tx antenna delay isn't stored in AON, so we'll do it ourselves.
    pub(super) tx_antenna_delay: Duration,
}

/// Any state struct that implements this trait signals that the radio is **not** sleeping.
pub trait Awake {}
impl Awake for Uninitialized {}
impl Awake for Ready {}
impl Awake for Sending {}
impl Awake for Receiving {}
/// Any state struct that implements this trait signals that the radio is sleeping.
pub trait Asleep {}
impl Asleep for Sleeping {}

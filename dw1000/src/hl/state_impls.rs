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

/// Indicates that the `DW1000` instance is currently receiving in single buffer mode (default)
#[derive(Debug)]
pub struct SingleBufferReceiving {
    finished: bool,
    config: RxConfig,
}

/// Indicates that the `DW1000` instance is currently receiving in double buffer mode
#[derive(Debug)]
pub struct DoubleBufferReceiving {
    finished: bool,
    config: RxConfig,
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
impl Awake for SingleBufferReceiving {}
impl Awake for DoubleBufferReceiving {}
/// Any state struct that implements this trait signals that the radio is sleeping.
pub trait Asleep {}
impl Asleep for Sleeping {}

/// Any state struct that implements this trait shares a number of rx operations
pub trait Receiving: Awake {
    fn mark_finished(&mut self);
    fn is_finished(&self) -> bool;
    fn get_rx_config(&self) -> &RxConfig;
}
impl Receiving for SingleBufferReceiving {
    fn mark_finished(&mut self) {
        self.finished = true;
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn get_rx_config(&self) -> &RxConfig {
        &self.config
    }
}
impl Receiving for DoubleBufferReceiving {
    fn mark_finished(&mut self) {
        self.finished = true;
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn get_rx_config(&self) -> &RxConfig {
        &self.config
    }
}

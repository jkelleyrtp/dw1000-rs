//! Implementation of double-sided two-way ranging
//!
//! This ranging technique is described in the DW1000 user manual, section 12.3.
//! This module uses three messages for a range measurement, as described in
//! section 12.3.2.
//!
//! This module defines the messages required, and provides code for sending and
//! decoding them. It is left to the user to tie all that together, by sending
//! out the messages at the right time.
//!
//! There can be some variation in the use of this module, depending on the use
//! case. Here is one example of how this module can be used:
//! 1. Nodes are divided into anchors and tags. Tags are those nodes whose
//!    position interests us. Anchors are placed in known locations to enable
//!    range measurements.
//! 2. Anchors regularly send out pings ([`Ping`]).
//! 3. Tags listen for these pings, and reply with a ranging request
//!    ([`Request`]) for each ping they receive.
//! 4. When an anchor receives a ranging request, it replies with a ranging
//!    response ([`Response`]).
//! 5. Once the tag receives the ranging response, it has all the information it
//!    needs to compute the distance.
//!
//! Please refer to the [examples] in the DWM1001 Board Support Crate for an
//! implementation of this scheme.
//!
//! In this scheme, anchors initiate the exchange, which results in the tag
//! having the distance information. Possible variations include the tag
//! initiating the request and the anchor calculating the distance, or a
//! peer-to-peer scheme without dedicated tags and anchors.
//!
//! Please note that using the code in this module without further processing of
//! the result will yield imprecise measurements. To improve the precision of
//! those measurements, a range bias needs to be applied. Please refer to the
//! user manual, and [this DWM1001 issue] for more information.
//!
//! [`Ping`]: struct.Ping.html
//! [`Request`]: struct.Request.html
//! [`Response`]: struct.Response.html
//! [examples]: https://github.com/braun-robotics/rust-dwm1001/tree/master/examples
//! [this DWM1001 issue]: https://github.com/braun-robotics/rust-dwm1001/issues/55


use core::mem::size_of;

use embedded_hal::{
    blocking::spi,
    digital::v2::OutputPin,
};
use serde::{
    Deserialize,
    Serialize,
};
use ssmarshal;

use crate::{hl, mac, time::{
    Duration,
    Instant,
}, DW1000, Error, Ready, Sending, TxConfig};


/// The transmission delay
///
/// This defines the transmission delay as 10 ms. This should be enough to
/// finish the rest of the preparation and send the message, even if we're
/// running with unoptimized code.
const TX_DELAY: u32 = 10_000_000;


/// Implemented by all ranging messages
pub trait Message: Sized + for<'de> Deserialize<'de> + Serialize {
    /// A prelude that identifies the message
    const PRELUDE: Prelude;

    /// The length of the message's prelude
    ///
    /// This is a bit of a hack that we need until `slice::<impl [T]>::len` is
    /// stable as a const fn.
    const PRELUDE_LEN: usize;

    /// The length of the whole message, including prelude and data
    const LEN: usize = Self::PRELUDE_LEN + size_of::<Self>();

    /// Decodes a received message of this type
    ///
    /// The user is responsible for receiving a message using
    /// [`DW1000::receive`]. Once a message has been received, this method can
    /// be used to check what type of message this is.
    ///
    /// Returns `Ok(None)`, if the message is not of the right type. Otherwise,
    /// returns `Ok(Some(RxMessage<Self>)), if the message is of the right type,
    /// and no error occured.
    fn decode<SPI, CS>(message: &hl::Message)
        -> Result<Option<RxMessage<Self>>, Error<SPI, CS>>
        where
            SPI: spi::Transfer<u8> + spi::Write<u8>,
            CS:  OutputPin,
    {
        if !message.frame.payload.starts_with(Self::PRELUDE.0) {
            // Not a message of this type
            return Ok(None);
        }

        if message.frame.payload.len() != Self::LEN {
            // Invalid message
            return Err(Error::BufferTooSmall {
                required_len: Self::LEN,
            });
        }

        // The message passes muster. Let's decode it.
        let (payload, _) = ssmarshal::deserialize::<Self>(
            &message.frame.payload[Self::PRELUDE.0.len()..
        ])?;

        Ok(Some(RxMessage {
            rx_time: message.rx_time,
            source:  message.frame.header.source,
            payload,
        }))
    }
}


/// An incoming ranging message
///
/// Contains the received payload, as well as some metadata that's required to
/// create a reply to the message.
#[derive(Debug)]
pub struct RxMessage<T: Message> {
    /// The time the message was received
    pub rx_time: Instant,

    /// The source of the message
    pub source: mac::Address,

    /// The message data
    pub payload: T,
}


/// An outgoing ranging message
///
/// Contains the payload to be sent, as well as some metadata.
#[derive(Debug)]
pub struct TxMessage<T: Message> {
    /// The recipient of the message
    ///
    /// This is an IEEE 802.15.4 MAC address. This could be a broadcast address,
    /// for messages that are sent to all other nodes in range.
    pub recipient: mac::Address,

    /// The time this message is going to be sent
    ///
    /// When creating this struct, this is going to be an instant in the near
    /// future. When sending the message, the sending is delayed to make sure it
    /// it sent at exactly this instant.
    pub tx_time: Instant,

    /// The actual message payload
    pub payload: T,
}

impl<T> TxMessage<T> where T: Message {
    /// Send this message via the DW1000
    ///
    /// Serializes the message payload and uses [`DW1000::send`] internally to
    /// send it.
    pub fn send<'r, SPI, CS>(&self, dw1000: DW1000<SPI, CS, Ready>)
        -> Result<DW1000<SPI, CS, Sending>, Error<SPI, CS>>
        where
            SPI: spi::Transfer<u8> + spi::Write<u8>,
            CS:  OutputPin,
    {
        // Create a buffer that fits the biggest message currently implemented.
        // This is a really ugly hack. The size of the buffer should just be
        // `T::LEN`. Unfortunately that's not possible. See:
        // https://github.com/rust-lang/rust/issues/42863
        const LEN: usize = 48;
        assert!(T::LEN <= LEN);
        let mut buf = [0; LEN];

        buf[..T::PRELUDE.0.len()].copy_from_slice(T::PRELUDE.0);
        ssmarshal::serialize(
            &mut buf[T::PRELUDE.0.len()..],
            &self.payload,
        )?;

        let future = dw1000.send(
            &buf[..T::LEN],
            self.recipient,
            Some(self.tx_time),
            TxConfig::default()
        )?;

        Ok(future)
    }
}


/// Sent before a message's data to identify the message
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Prelude(pub &'static [u8]);


/// Ranging ping message
///
/// This message is typically sent to initiate a range measurement transaction.
/// See [module documentation] for more info.
///
/// [module documentation]: index.html
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Ping {
    /// When the ping was sent, in local sender time
    pub ping_tx_time: Instant,
}

impl Ping {
    /// Creates a new ping message
    ///
    /// Only creates the message, but doesn't yet send it. Sets the transmission
    /// time to 10 milliseconds in the future. Make sure to send the message
    /// within that time frame, or the distance measurement will be negatively
    /// affected.
    pub fn new<SPI, CS>(dw1000: &mut DW1000<SPI, CS, Ready>)
        -> Result<TxMessage<Self>, Error<SPI, CS>>
        where
            SPI: spi::Transfer<u8> + spi::Write<u8>,
            CS:  OutputPin,
    {
        let tx_time = dw1000.sys_time()? + Duration::from_nanos(TX_DELAY);
        let ping_tx_time = tx_time + dw1000.get_tx_antenna_delay()?;

        let payload = Ping {
            ping_tx_time,
        };

        Ok(TxMessage {
            recipient: mac::Address::broadcast(&mac::AddressMode::Short),
            tx_time,
            payload,
        })
    }
}

impl Message for Ping {
    const PRELUDE:     Prelude = Prelude(b"RANGING PING");
    const PRELUDE_LEN: usize   = 12;
}


/// Ranging request message
///
/// This message is typically sent in response to a ranging ping, to request a
/// ranging response. See [module documentation] for more info.
///
/// [module documentation]: index.html
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Request {
    /// When the original ping was sent, in local time on the anchor
    pub ping_tx_time: Instant,

    /// The time between the ping being received and the reply being sent
    pub ping_reply_time: Duration,

    /// When the ranging request was sent, in local sender time
    pub request_tx_time: Instant,
}

impl Request {
    /// Creates a new ranging request message
    ///
    /// Only creates the message, but doesn't yet send it. Sets the transmission
    /// time to 10 milliseconds in the future. Make sure to send the message
    /// within that time frame, or the distance measurement will be negatively
    /// affected.
    pub fn new<SPI, CS>(
        dw1000: &mut DW1000<SPI, CS, Ready>,
        ping:   &RxMessage<Ping>,
    )
        -> Result<TxMessage<Self>, Error<SPI, CS>>
        where
            SPI: spi::Transfer<u8> + spi::Write<u8>,
            CS:  OutputPin,
    {
        let tx_time = dw1000.sys_time()? + Duration::from_nanos(TX_DELAY);
        let request_tx_time = tx_time + dw1000.get_tx_antenna_delay()?;

        let ping_reply_time = request_tx_time.duration_since(ping.rx_time);

        let payload = Request {
            ping_tx_time: ping.payload.ping_tx_time,
            ping_reply_time,
            request_tx_time,
        };

        Ok(TxMessage {
            recipient: ping.source,
            tx_time,
            payload,
        })
    }
}

impl Message for Request {
    const PRELUDE:     Prelude = Prelude(b"RANGING REQUEST");
    const PRELUDE_LEN: usize   = 15;
}


/// Ranging response message
///
/// This message is typically sent in response to a ranging request, to wrap up
/// the range measurement transaction.. See [module documentation] for more
/// info.
///
/// [module documentation]: index.html
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Response {
    /// The time between the ping being received and the reply being sent
    pub ping_reply_time: Duration,

    /// The time between the ping being sent and the reply being received
    pub ping_round_trip_time: Duration,

    /// The time the ranging request was sent, in local sender time
    pub request_tx_time: Instant,

    /// The time between the request being received and a reply being sent
    pub request_reply_time: Duration,
}

impl Response {
    /// Creates a new ranging response message
    ///
    /// Only creates the message, but doesn't yet send it. Sets the transmission
    /// time to 10 milliseconds in the future. Make sure to send the message
    /// within that time frame, or the distance measurement will be negatively
    /// affected.
    pub fn new<SPI, CS>(
        dw1000:  &mut DW1000<SPI, CS, Ready>,
        request: &RxMessage<Request>,
    )
        -> Result<TxMessage<Self>, Error<SPI, CS>>
        where
            SPI: spi::Transfer<u8> + spi::Write<u8>,
            CS:  OutputPin,
    {
        let tx_time = dw1000.sys_time()? + Duration::from_nanos(TX_DELAY);
        let response_tx_time = tx_time + dw1000.get_tx_antenna_delay()?;

        let ping_round_trip_time =
            request.rx_time.duration_since(request.payload.ping_tx_time);
        let request_reply_time =
            response_tx_time.duration_since(request.rx_time);

        let payload = Response {
            ping_reply_time: request.payload.ping_reply_time,
            ping_round_trip_time,
            request_tx_time: request.payload.request_tx_time,
            request_reply_time,
        };

        Ok(TxMessage {
            recipient: request.source,
            tx_time,
            payload,
        })
    }
}

impl Message for Response {
    const PRELUDE:     Prelude = Prelude(b"RANGING RESPONSE");
    const PRELUDE_LEN: usize   = 16;
}


/// Computes the distance to another node from a ranging response
pub fn compute_distance_mm(response: &RxMessage<Response>)
    -> Result<u64, ComputeDistanceError>
{
    // To keep variable names to a reasonable length, this function uses `rt` as
    // a short-hand for "reply time" and `rtt` and a short-hand for "round-trip
    // time".

    let ping_rt = response.payload.ping_reply_time.value();
    let ping_rtt = response.payload.ping_round_trip_time.value();
    let request_rt = response.payload.request_reply_time.value();
    let request_rtt = response.rx_time
        .duration_since(response.payload.request_tx_time)
        .value();

    // Compute time of flight according to the formula given in the DW1000 user
    // manual, section 12.3.2.
    let rtt_product = ping_rtt * request_rtt;
    let rt_product = ping_rt * request_rt;
    let sum = ping_rtt + request_rtt + ping_rt + request_rt;
    let time_of_flight = (rtt_product - rt_product) / sum;

    // Nominally, all time units are based on a 64 Ghz clock, meaning each time
    // unit is 1/64 ns.

    const SPEED_OF_LIGHT: u64 = 299_792_458; // m/s or nm/ns

    let distance_nm_times_64 = SPEED_OF_LIGHT.checked_mul(time_of_flight)
        .ok_or(ComputeDistanceError::TimeOfFlightTooLarge)?;
    let distance_mm          = distance_nm_times_64 / 64 / 1_000_000;

    Ok(distance_mm)
}


/// Returned from [`compute_distance_mm`] in case of an error
#[derive(Debug)]
pub enum ComputeDistanceError {
    /// The time of flight is so large, the distance calculation would overflow
    TimeOfFlightTooLarge,
}

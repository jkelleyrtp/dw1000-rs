//! Supports double-sided two-way ranging
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
//! In this scheme, anchors initiate the exchange, which results in the tag
//! having the distance information. Possible variations include the tag
//! initiating the request and the anchor calculating the distance, or a
//! peer-to-peer scheme without dedicated tags and anchors.


use core::mem::size_of;

use hal::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use ssmarshal;

use ::{
    hl,
    mac,
    util,
    Duration,
    DW1000,
    Error,
    Instant,
    Ready,
    TxFuture,
};


/// The transmission delay
///
/// This defines the transmission delay as 10 ms. This should be enough to
/// finish the rest of the preparation and send the message, even if we're
/// running with unoptimized code.
const TX_DELAY: u32 = 10_000_000;


/// A ranging message
pub trait Message: Sized {
    /// A prelude that identifies the message
    const PRELUDE: Prelude;

    /// The length of the message's prelude
    ///
    /// This is a bit of a hack that we need until `slice::<impl [T]>::len` is
    /// stable as a const fn.
    const PRELUDE_LEN: usize;

    /// The length of the whole message, including prelude and data
    const LEN: usize = Self::PRELUDE_LEN + size_of::<Self::Data>();

    /// The message data
    type Data: for<'de> Deserialize<'de> + Serialize;

    /// Returns this message's data
    fn data(&self) -> &Self::Data;

    /// Returns this message's recipient
    fn recipient(&self) -> mac::Address;

    /// Returns the transmission time of this message
    fn tx_time(&self) -> Instant;

    /// Send this message
    fn send<'r, SPI>(&self, dw1000: &'r mut DW1000<SPI, Ready>)
        -> Result<TxFuture<'r, SPI>, Error>
        where SPI: SpimExt
    {
        // Create a buffer that fits the biggest message currently implemented.
        // This is a really ugly hack. The size of the buffer should just be
        // `Self::LEN`. Unfortunately that's not possible. See:
        // https://github.com/rust-lang/rust/issues/42863
        const LEN: usize = 48;
        assert!(Self::LEN <= LEN);
        let mut buf = [0; LEN];

        buf[..Self::PRELUDE.0.len()].copy_from_slice(Self::PRELUDE.0);
        ssmarshal::serialize(&mut buf[Self::PRELUDE.0.len()..], self.data())?;

        let future = dw1000.send(
            &buf[..Self::LEN],
            self.recipient(),
            Some(self.tx_time()),
        )?;

        Ok(future)
    }

    /// Decodes a received message of this type
    fn decode(message: &hl::Message) -> Result<Option<RxMessage<Self>>, Error> {
        if !message.frame.payload.starts_with(Self::PRELUDE.0) {
            // Not a request of this type
            return Ok(None);
        }

        if message.frame.payload.len() != Self::LEN {
            // Invalid request
            return Err(Error::BufferTooSmall {
                required_len: Self::LEN,
            });
        }

        // The request passes muster. Let's decode it.
        let (data, _) = ssmarshal::deserialize::<Self::Data>(
            &message.frame.payload[Self::PRELUDE.0.len()..
        ])?;

        Ok(Some(RxMessage {
            rx_time: message.rx_time,
            source:  message.frame.header.source,
            data,
        }))
    }
}


/// A received ranging message
///
/// Contains the received data, as well as some metadata that's required to
/// create a reply to the message.
pub struct RxMessage<T: Message> {
    /// The time the message was received
    pub rx_time: Instant,

    /// The source of the message
    pub source: mac::Address,

    /// The message data
    pub data: T::Data,
}


/// Sent before a message's data to identify the message
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Prelude(pub &'static [u8]);


/// Ranging ping message
#[derive(Debug)]
pub struct Ping {
    tx_time: Instant,
    data:    PingData,
}

/// A ranging ping
///
/// Sent out regularly by anchors.
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct PingData {
    /// When the ping was sent, in local sender time
    pub ping_tx_time: Instant,
}

impl Ping {
    /// Creates a new ping message
    pub fn initiate<SPI>(dw1000: &mut DW1000<SPI, Ready>) -> Result<Self, Error>
        where SPI: SpimExt
    {
        let tx_antenna_delay = dw1000.get_tx_antenna_delay()?;
        let tx_time          = dw1000.time_from_delay(TX_DELAY)?;

        let data = PingData {
            ping_tx_time: tx_time + tx_antenna_delay,
        };

        Ok(Ping {
            tx_time,
            data,
        })
    }
}

impl Message for Ping {
    const PRELUDE:     Prelude = Prelude(b"RANGING PING");
    const PRELUDE_LEN: usize   = 12;

    type Data = PingData;

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn recipient(&self) -> mac::Address {
        mac::Address::broadcast()
    }

    fn tx_time(&self) -> Instant {
        self.tx_time
    }
}



/// Ranging request message
#[derive(Debug)]
pub struct Request {
    recipient: mac::Address,
    tx_time:   Instant,
    data:      RequestData,
}

/// A ranging request
///
/// Sent by tags in response to a ping.
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct RequestData {
    /// When the original ping was sent, in local time on the anchor
    pub ping_tx_time: Instant,

    /// The time between the ping being received and the reply being sent
    pub ping_reply_time: Duration,

    /// When the ranging request was sent, in local sender time
    pub request_tx_time: Instant,
}

impl Request {
    /// Creates a new ranging request message
    pub fn initiate<SPI>(
        dw1000: &mut DW1000<SPI, Ready>,
        ping:   RxMessage<Ping>,
    )
        -> Result<Self, Error>
        where SPI: SpimExt
    {
        let tx_antenna_delay = dw1000.get_tx_antenna_delay()?;
        let tx_time          = dw1000.time_from_delay(TX_DELAY)?;
        let request_tx_time  = tx_time + tx_antenna_delay;

        let ping_reply_time = util::duration_between(
            ping.rx_time,
            request_tx_time,
        );

        let data = RequestData {
            ping_tx_time: ping.data.ping_tx_time,
            ping_reply_time,
            request_tx_time,
        };

        Ok(Request {
            recipient: ping.source,
            tx_time,
            data,
        })
    }
}

impl Message for Request {
    const PRELUDE:     Prelude = Prelude(b"RANGING REQUEST");
    const PRELUDE_LEN: usize   = 15;

    type Data = RequestData;

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn recipient(&self) -> mac::Address {
        self.recipient
    }

    fn tx_time(&self) -> Instant {
        self.tx_time
    }
}


/// A ranging response
///
/// Sent by anchors in response to a ranging request.
#[derive(Debug)]
pub struct Response {
    recipient: mac::Address,
    tx_time:   Instant,
    data:      ResponseData,
}

/// Ranging response data
#[derive(Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct ResponseData {
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
    pub fn initiate<SPI>(
        dw1000:  &mut DW1000<SPI, Ready>,
        request: RxMessage<Request>,
    )
        -> Result<Self, Error>
        where SPI: SpimExt
    {
        let tx_antenna_delay = dw1000.get_tx_antenna_delay()?;
        let tx_time          = dw1000.time_from_delay(TX_DELAY)?;
        let response_tx_time = tx_time + tx_antenna_delay;

        let ping_round_trip_time = util::duration_between(
            request.data.ping_tx_time,
            request.rx_time,
        );
        let request_reply_time = util::duration_between(
            request.rx_time,
            response_tx_time,
        );

        let data = ResponseData {
            ping_reply_time: request.data.ping_reply_time,
            ping_round_trip_time,
            request_tx_time: request.data.request_tx_time,
            request_reply_time,
        };

        Ok(Response {
            recipient: request.source,
            tx_time,
            data,
        })
    }
}

impl Message for Response {
    const PRELUDE:     Prelude = Prelude(b"RANGING RESPONSE");
    const PRELUDE_LEN: usize   = 16;

    type Data = ResponseData;

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn recipient(&self) -> mac::Address {
        self.recipient
    }

    fn tx_time(&self) -> Instant {
        self.tx_time
    }
}


/// Computes the distance to another node from a ranging response
///
/// Returns `None`, if the computed time of flight is so large the distance
/// calculation would overflow.
pub fn compute_distance_mm(response: &RxMessage<Response>) -> Option<u64> {
    let request_round_trip_time = util::duration_between(
        response.data.request_tx_time,
        response.rx_time,
    );

    // Compute time of flight according to the formula given in the DW1000 user
    // manual, section 12.3.2.
    let rtt_product =
        response.data.ping_round_trip_time.0 *
        request_round_trip_time.0;
    let reply_time_product =
        response.data.ping_reply_time.0 *
        response.data.request_reply_time.0;
    let complete_sum =
        response.data.ping_round_trip_time.0 +
        request_round_trip_time.0 +
        response.data.ping_reply_time.0 +
        response.data.request_reply_time.0;
    let time_of_flight = (rtt_product - reply_time_product) / complete_sum;

    // Nominally, all time units are based on a 64 Ghz clock, meaning each time
    // unit is 1/64 ns.

    const SPEED_OF_LIGHT: u64 = 299_792_458; // m/s or nm/ns

    let distance_nm_times_64 = SPEED_OF_LIGHT.checked_mul(time_of_flight)?;
    let distance_mm          = distance_nm_times_64 / 64 / 1_000_000;

    Some(distance_mm)
}

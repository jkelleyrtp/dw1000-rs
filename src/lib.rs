//! Driver crate for the DW1000 UWB transceiver


#![no_std]

#![deny(missing_docs)]
#![deny(warnings)]


extern crate nrf52_hal;


use nrf52_hal::{
    prelude::*,
    gpio::{
        p0,
        Output,
        PushPull,
    },
    spim,
    Spim,
};


/// Entry point to the DW1000 driver API
pub struct DW1000<SPI> {
    spim       : Spim<SPI>,
    chip_select: p0::P0_Pin<Output<PushPull>>,
}

impl<SPI> DW1000<SPI> where SPI: SpimExt {
    /// Create a new instance of `DW1000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW1000.
    pub fn new(
        spim       : Spim<SPI>,
        chip_select: p0::P0_Pin<Output<PushPull>>
    )
        -> Self
    {
        DW1000 {
            spim,
            chip_select,
        }
    }

    /// Read from a register
    pub fn read<R>(&mut self) -> Result<R::Read, spim::Error>
        where
            R: Register + Readable,
    {
        let mut tx_buffer = [0; 1];
        make_header::<R>(false, &mut tx_buffer);

        let mut r = R::read();

        self.spim.read(
            &mut self.chip_select,
            &tx_buffer,
            R::buffer(&mut r),
        )?;

        Ok(r)
    }

    /// Write to a register
    pub fn write<R, F>(&mut self, f: F) -> Result<(), spim::Error>
        where
            R: Register + Writable,
            F: FnOnce(&mut R::Write) -> &mut R::Write,
    {
        let mut w = R::write();
        f(&mut w);
        let tx_buffer = R::buffer(&mut w);
        make_header::<R>(true, tx_buffer);

        self.spim.write(&mut self.chip_select, &tx_buffer)?;

        Ok(())
    }

    /// Modify a register
    pub fn modify<R, F>(&mut self, f: F) -> Result<(), spim::Error>
        where
            R: Register + Readable + Writable,
            F: for<'r>
                FnOnce(&mut R::Read, &'r mut R::Write) -> &'r mut R::Write,
    {
        let mut r = self.read::<R>()?;
        let mut w = R::write();

        <R as Writable>::buffer(&mut w)
            .copy_from_slice(<R as Readable>::buffer(&mut r));

        f(&mut r, &mut w);

        let tx_buffer = <R as Writable>::buffer(&mut w);
        make_header::<R>(true, tx_buffer);

        self.spim.write(&mut self.chip_select, &tx_buffer)?;

        Ok(())
    }
}


/// Initializes the header for a register in the given buffer
fn make_header<R: Register>(write: bool, buffer: &mut [u8]) {
    buffer[0] =
        ((write as u8) << 7 & 0x80) |
        (0             << 6 & 0x40) |  // no sub-index
        (R::ID              & 0x3f);
}


/// Implemented for all registers
///
/// The DW1000 user manual, section 7.1, specifies what the values of the
/// constant should be for each register.
pub trait Register {
    /// The register index
    const ID:  u8;

    /// The lenght of the register
    const LEN: usize;
}

/// Marker trait for registers that can be read
pub trait Readable {
    /// The type that is used to read from the register
    type Read;

    /// Return the read type for this register
    fn read() -> Self::Read;

    /// Return the read type's internal buffer
    fn buffer(r: &mut Self::Read) -> &mut [u8];
}

/// Marker trait for registers that can be written
pub trait Writable {
    /// The type that is used to write to the register
    type Write;

    /// Return the write type for this register
    fn write() -> Self::Write;

    /// Return the write type's internal buffer
    fn buffer(w: &mut Self::Write) -> &mut [u8];
}

macro_rules! impl_register {
    (
        $(
            $id:expr,
            $len:expr,
            $rw:tt,
            $name:ident($name_lower:ident) {
            #[$doc:meta]
            $(
                $field:ident,
                $first_bit:expr,
                $last_bit:expr,
                $ty:ty;
                #[$field_doc:meta]
            )*
            }
        )*
    ) => {
        $(
            #[$doc]
            #[allow(non_camel_case_types)]
            pub struct $name;

            impl Register for $name {
                const ID:  u8    = $id;
                const LEN: usize = $len;
            }

            #[$doc]
            pub mod $name_lower {
                /// Used to read from the register
                pub struct R(pub(crate) [u8; $len + 1]);

                impl R {
                    $(
                        #[$field_doc]
                        pub fn $field(&self) -> $ty {
                            use core::mem::size_of;
                            use FromBytes;

                            // Get all bytes that contain our field. The field
                            // might fill out these bytes completely, or only
                            // some bits in them.
                            const START: usize = $first_bit / 8;
                            const END:   usize = $last_bit  / 8 + 1;
                            let mut bytes = [0; END - START];
                            bytes.copy_from_slice(&self.0[START+1 .. END+1]);

                            // Before we can convert the field into a number and
                            // return it, we need to shift it, to make sure
                            // there are no other bits to the right of it. Let's
                            // start by determining the offset of the field
                            // within a byte.
                            const OFFSET_IN_BYTE: usize = $first_bit % 8;

                            if OFFSET_IN_BYTE > 0 {
                                // Shift the first byte. We always have at least
                                // one byte here, so this always works.
                                bytes[0] >>= OFFSET_IN_BYTE;

                                // If there are more bytes, let's shift those
                                // too.
                                // We need to allow exceeding bitshifts in this
                                // loop, as we run into that if `OFFSET_IN_BYTE`
                                // equals `0`. Please note that we never
                                // actually encounter that at runtime, due to
                                // the if condition above.
                                let mut i = 1;
                                #[allow(exceeding_bitshifts)]
                                while i < bytes.len() {
                                    bytes[i - 1] |=
                                        bytes[i] << 8 - OFFSET_IN_BYTE;
                                    bytes[i] >>= OFFSET_IN_BYTE;
                                    i += 1;
                                }
                            }

                            // If the field didn't completely fill out its last
                            // byte, we might have bits from unrelated fields
                            // there. Let's erase those before doing the final
                            // conversion into the field's data type.
                            const BITS_ABOVE_FIELD: usize =
                                8 - (($last_bit - $first_bit + 1) % 8);
                            const LAST_INDEX: usize = size_of::<$ty>() - 1;
                            if BITS_ABOVE_FIELD < 8 {
                                // Need to allow exceeding bitshifts to make the
                                // compiler happy. They're never actually
                                // encountered at runtime, due to the if
                                // condition.
                                #[allow(exceeding_bitshifts)]
                                {
                                    bytes[LAST_INDEX] <<= BITS_ABOVE_FIELD;
                                    bytes[LAST_INDEX] >>= BITS_ABOVE_FIELD;
                                }
                            }

                            // Now all that's left is to convert the bytes into
                            // the field's type. Please note that methods for
                            // converting numbers to/from bytes are coming to
                            // stable Rust, so we might be able to remove our
                            // custom infrastructure here. Tracking issue:
                            // https://github.com/rust-lang/rust/issues/52963
                            <$ty as FromBytes>::from_bytes(&bytes)
                        }
                    )*
                }

                /// Used to write to the register
                pub struct W(pub(crate) [u8; $len + 1]);

                impl W {
                    $(
                        #[$field_doc]
                        pub fn $field(&mut self, value: $ty) -> &mut Self {
                            use ToBytes;

                            // Convert value into bytes
                            let source = <$ty as ToBytes>::to_bytes(value);

                            // Now, let's figure out where the bytes are located
                            // within the register array.
                            const START:          usize = $first_bit / 8;
                            const END:            usize = $last_bit  / 8 + 1;
                            const OFFSET_IN_BYTE: usize = $first_bit % 8;

                            // Also figure out the length of the value in bits.
                            // That's going to come in handy.
                            const LEN: usize = $last_bit - $first_bit + 1;


                            // We need to track how many bits are left in the
                            // value overall, and in the value's current byte.
                            let mut bits_left         = LEN;
                            let mut bits_left_in_byte = 8;

                            // We also need to track how many bits have already
                            // been written to the current target byte.
                            let mut bits_written_to_byte = 0;

                            // Now we can take the bytes from the value, shift
                            // them, mask them, and write them into the target
                            // array.
                            let mut source_i  = 0;
                            let mut target_i  = START;
                            while target_i < END {
                                // Values don't always end at byte boundaries,
                                // so we need to mask the bytes when writing to
                                // the slice.
                                // Let's start out assuming we can write to the
                                // whole byte of the slice. This will be true
                                // for the middle bytes of our value.
                                let mut mask = 0xff;

                                // Let's keep track of the offset we're using to
                                // write to this byte. We're going to need it.
                                let mut offset_in_this_byte = 0;

                                // If this is the first byte we're writing to
                                // the slice, we need to remove the lower bits
                                // of the mask.
                                if target_i == START {
                                    mask <<= OFFSET_IN_BYTE;
                                    offset_in_this_byte = OFFSET_IN_BYTE;
                                }

                                // If this is the last byte we're writing to the
                                // slice, we need to remove the higher bits of
                                // the mask. Please note that we could be
                                // writing to _both_ the first and the last
                                // byte.
                                if target_i == END - 1 {
                                    let shift =
                                        8 - bits_left - offset_in_this_byte;
                                    mask <<= shift;
                                    mask >>= shift;
                                }

                                mask <<= bits_written_to_byte;

                                // Read the value from `source`
                                let value = source[source_i]
                                    >> 8 - bits_left_in_byte
                                    << offset_in_this_byte
                                    << bits_written_to_byte;

                                // Zero the target bits in the slice, then write
                                // the value.
                                self.0[target_i + 1] &= !mask;
                                self.0[target_i + 1] |= value & mask;

                                // The number of bits that were expected to be
                                // written to the target byte.
                                let bits_needed = mask.count_ones() as usize;

                                // The number of bits we actually wrote to the
                                // target byte.
                                let bits_used = bits_needed.min(
                                    bits_left_in_byte - offset_in_this_byte
                                );

                                bits_left -= bits_used;
                                bits_written_to_byte += bits_used;

                                // Did we use up all the bits in the source
                                // byte? If so, we can move on to the next one.
                                if bits_left_in_byte > bits_used {
                                    bits_left_in_byte -= bits_used;
                                }
                                else {
                                    bits_left_in_byte =
                                        8 - (bits_used - bits_left_in_byte);

                                    source_i += 1;
                                }

                                // Did we write all the bits in the target byte?
                                // If so, we can move on to the next one.
                                if bits_used == bits_needed {
                                    target_i += 1;
                                    bits_written_to_byte = 0;
                                }
                            }

                            self
                        }
                    )*
                }
            }

            impl_rw!($rw, $name, $name_lower, $len);
        )*
    }
}

macro_rules! impl_rw {
    (RO, $name:ident, $name_lower:ident, $len:expr) => {
        impl_rw!(@R, $name, $name_lower, $len);
    };
    (RW, $name:ident, $name_lower:ident, $len:expr) => {
        impl_rw!(@R, $name, $name_lower, $len);
        impl_rw!(@W, $name, $name_lower, $len);
    };

    (@R, $name:ident, $name_lower:ident, $len:expr) => {
        impl Readable for $name {
            type Read = $name_lower::R;

            fn read() -> Self::Read {
                $name_lower::R([0; $len + 1])
            }

            fn buffer(r: &mut Self::Read) -> &mut [u8] {
                &mut r.0
            }
        }
    };
    (@W, $name:ident, $name_lower:ident, $len:expr) => {
        impl Writable for $name {
            type Write = $name_lower::W;

            fn write() -> Self::Write {
                $name_lower::W([0; $len + 1])
            }

            fn buffer(w: &mut Self::Write) -> &mut [u8] {
                &mut w.0
            }
        }
    };
}

impl_register! {
    0x00, 4, RO, DEV_ID(dev_id) {         /// Device identifier
        rev,     0,  3, u8;               /// Revision
        ver,     4,  7, u8;               /// Version
        model,   8, 15, u8;               /// Model
        ridtag, 16, 31, u16;              /// Register Identification Tag
    }
    0x01, 8, RW, EUI(eui) {               /// Extended Unique Identifier
        eui, 0, 63, u64;                  /// Extended Unique Identifier
    }
    0x03, 4, RW, PANADR(panadr) {         /// PAN Identifier and Short Address
        short_addr,  0, 15, u16;          /// Short Address
        pan_id,     16, 31, u16;          /// PAN Identifier
    }
    0x08, 5, RW, TX_FCTRL(tx_fctrl) {     /// TX Frame Control
        tflen,     0,  6, u8;             /// TX Frame Length
        tfle,      7,  9, u8;             /// TX Frame Length Extension
        txbr,     13, 14, u8;             /// TX Bit Rate
        tr,       15, 15, u8;             /// TX Ranging Enable
        txprf,    16, 17, u8;             /// TX Pulse Repetition Frequency
        txpsr,    18, 19, u8;             /// TX Preamble Symbol Repetitions
        pe,       20, 21, u8;             /// Preamble Extension
        txboffs,  22, 31, u16;            /// TX Buffer Index Offset
        ifsdelay, 32, 39, u8;             /// Inter-Frame Spacing
    }
    0x0D, 4, RW, SYS_CTRL(sys_ctrl) {     /// System Control Register
        sfcst,      0,  0, u8;            /// Suppress Auto-FCS Transmission
        txstrt,     1,  1, u8;            /// Transmit Start
        txdlys,     2,  2, u8;            /// Transmitter Delayed Sending
        cansfcs,    3,  3, u8;            /// Cancel Auto-FCS Suppression
        trxoff,     6,  6, u8;            /// Transceiver Off
        wait4resp,  7,  7, u8;            /// Wait for Response
        rxenab,     8,  8, u8;            /// Enable Receiver
        rxdlye,     9,  9, u8;            /// Receiver Delayed Enable
        hrbpt,     24, 24, u8;            /// Host Side RX Buffer Pointer Toggle
    }
    0x0F, 5, RW, SYS_STATUS(sys_status) { /// System Event Status Register
        irqs,       0,  0, u8;            /// Interrupt Request Status
        cplock,     1,  1, u8;            /// Clock PLL Lock
        esyncr,     2,  2, u8;            /// External Sync Clock Reset
        aat,        3,  3, u8;            /// Automatic Acknowledge Trigger
        txfrb,      4,  4, u8;            /// TX Frame Begins
        txprs,      5,  5, u8;            /// TX Preamble Sent
        txphs,      6,  6, u8;            /// TX PHY Header Sent
        txfrs,      7,  7, u8;            /// TX Frame Sent
        rxprd,      8,  8, u8;            /// RX Preamble Detected
        rxsfdd,     9,  9, u8;            /// RX SFD Detected
        ldedone,   10, 10, u8;            /// LDE Processing Done
        rxphd,     11, 11, u8;            /// RX PHY Header Detect
        rxphe,     12, 12, u8;            /// RX PHY Header Error
        rxdfr,     13, 13, u8;            /// RX Data Frame Ready
        rxfcg,     14, 14, u8;            /// RX FCS Good
        rxfce,     15, 15, u8;            /// RX FCS Error
        rxrfsl,    16, 16, u8;            /// RX Reed-Solomon Frame Sync Loss
        rxrfto,    17, 17, u8;            /// RX Frame Wait Timeout
        ldeerr,    18, 18, u8;            /// Leading Edge Detection Error
        rxovrr,    20, 20, u8;            /// RX Overrun
        rxpto,     21, 21, u8;            /// Preamble Detection Timeout
        gpioirq,   22, 22, u8;            /// GPIO Interrupt
        slp2init,  23, 23, u8;            /// SLEEP to INIT
        rfpll_ll,  24, 24, u8;            /// RF PLL Losing Lock
        clkpll_ll, 25, 25, u8;            /// Clock PLL Losing Lock
        rxsfdto,   26, 26, u8;            /// Receive SFD Timeout
        hpdwarn,   27, 27, u8;            /// Half Period Delay Warning
        txberr,    28, 28, u8;            /// TX Buffer Error
        affrej,    29, 29, u8;            /// Auto Frame Filtering Rejection
        hsrbp,     30, 30, u8;            /// Host Side RX Buffer Pointer
        icrbp,     31, 31, u8;            /// IC Side RX Buffer Pointer
        rxrscs,    32, 32, u8;            /// RX Reed-Solomon Correction Status
        rxprej,    33, 33, u8;            /// RX Preamble Rejection
        txpute,    34, 34, u8;            /// TX Power Up Time Error
    }
}


/// Transmit Data Buffer
///
/// Currently only the first 127 bytes of the buffer are supported, which is
/// enough to support standard Standard IEEE 802.15.4 UWB frames.
#[allow(non_camel_case_types)]
pub struct TX_BUFFER;

impl Register for TX_BUFFER {
    const ID:  u8    = 0x09;
    const LEN: usize = 127;
}

impl Writable for TX_BUFFER {
    type Write = tx_buffer::W;

    fn write() -> Self::Write {
        tx_buffer::W([0; 127 + 1])
    }

    fn buffer(w: &mut Self::Write) -> &mut [u8] {
        &mut w.0
    }
}


/// Transmit Data Buffer
pub mod tx_buffer {
    /// Used to write to the register
    pub struct W(pub(crate) [u8; 127 + 1]);

    impl W {
        /// Write data to the buffer
        ///
        /// `data` must at most be 127 bytes long.
        pub fn data(&mut self, data: &[u8]) -> &mut Self {
            self.0[1 .. data.len() + 1].copy_from_slice(data);
            self
        }
    }
}


trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl FromBytes for u8 {
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

impl FromBytes for u16 {
    fn from_bytes(bytes: &[u8]) -> Self {
        (bytes[1] as u16) << 8 |
        (bytes[0] as u16) << 0
    }
}

impl FromBytes for u32 {
    fn from_bytes(bytes: &[u8]) -> Self {
        (bytes[3] as u32) << 24 |
        (bytes[2] as u32) << 16 |
        (bytes[1] as u32) <<  8 |
        (bytes[0] as u32) <<  0
    }
}

impl FromBytes for u64 {
    fn from_bytes(bytes: &[u8]) -> Self {
        (bytes[7] as u64) << 56 |
        (bytes[6] as u64) << 48 |
        (bytes[5] as u64) << 40 |
        (bytes[4] as u64) << 32 |
        (bytes[3] as u64) << 24 |
        (bytes[2] as u64) << 16 |
        (bytes[1] as u64) <<  8 |
        (bytes[0] as u64) <<  0
    }
}


trait ToBytes {
    type Bytes;

    fn to_bytes(self) -> Self::Bytes;
}

impl ToBytes for u8 {
    type Bytes = [u8; 1];

    fn to_bytes(self) -> Self::Bytes {
        [self]
    }
}

impl ToBytes for u16 {
    type Bytes = [u8; 2];

    fn to_bytes(self) -> Self::Bytes {
        [
            ((self & 0x00ff) >> 0) as u8,
            ((self & 0xff00) >> 8) as u8,
        ]
    }
}

impl ToBytes for u32 {
    type Bytes = [u8; 4];

    fn to_bytes(self) -> Self::Bytes {
        [
            ((self & 0x000000ff) >>  0) as u8,
            ((self & 0x0000ff00) >>  8) as u8,
            ((self & 0x00ff0000) >> 16) as u8,
            ((self & 0xff000000) >> 24) as u8,
        ]
    }
}

impl ToBytes for u64 {
    type Bytes = [u8; 8];

    fn to_bytes(self) -> Self::Bytes {
        [
            ((self & 0x00000000000000ff) >>  0) as u8,
            ((self & 0x000000000000ff00) >>  8) as u8,
            ((self & 0x0000000000ff0000) >> 16) as u8,
            ((self & 0x00000000ff000000) >> 24) as u8,
            ((self & 0x000000ff00000000) >> 32) as u8,
            ((self & 0x0000ff0000000000) >> 40) as u8,
            ((self & 0x00ff000000000000) >> 48) as u8,
            ((self & 0xff00000000000000) >> 56) as u8,
        ]
    }
}

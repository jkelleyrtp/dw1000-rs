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
        let tx_buffer = [make_header::<R>(false)];

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
        tx_buffer[0] = make_header::<R>(true);

        self.spim.write(&mut self.chip_select, &tx_buffer)?;

        Ok(())
    }
}


fn make_header<R: Register>(write: bool) -> u8 {
    ((write as u8) << 7 & 0x80) |
    (0             << 6 & 0x40) |  // no sub-index
    (R::ID              & 0x3f)
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
    0x00, 4, RO, DEV_ID(dev_id) { /// Device identifier
        rev,     0,  3, u8;       /// Revision
        ver,     4,  7, u8;       /// Version
        model,   8, 15, u8;       /// Model
        ridtag, 16, 31, u16;      /// Register Identification Tag
    }
    0x01, 8, RW, EUI(eui) {       /// Extended Unique Identifier
        eui, 0, 63, u64;          /// Extended Unique Identifier
    }
    0x03, 4, RW, PANADR(panadr) { /// PAN Identifier and Short Address
        short_addr,  0, 15, u16;  /// Short Address
        pan_id,     16, 31, u16;  /// PAN Identifier
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


impl eui::W {
    /// Extended Unique Identifier
    pub fn eui(&mut self, value: u64) -> &mut Self {
        self.0[8] = ((value & 0xff00000000000000) >> 56) as u8;
        self.0[7] = ((value & 0x00ff000000000000) >> 48) as u8;
        self.0[6] = ((value & 0x0000ff0000000000) >> 40) as u8;
        self.0[5] = ((value & 0x000000ff00000000) >> 32) as u8;
        self.0[4] = ((value & 0x00000000ff000000) >> 24) as u8;
        self.0[3] = ((value & 0x0000000000ff0000) >> 16) as u8;
        self.0[2] = ((value & 0x000000000000ff00) >>  8) as u8;
        self.0[1] = (value & 0x00000000000000ff) as u8;

        self
    }
}

impl panadr::W {
    /// Short Address
    pub fn short_addr(mut self, value: u16) -> Self {
        self.0[2] = ((value & 0xff00) >> 8) as u8;
        self.0[1] = (value & 0x00ff) as u8;

        self
    }

    /// PAN Identifier
    pub fn pan_id(mut self, value: u16) -> Self {
        self.0[4] = ((value & 0xff00) >> 8) as u8;
        self.0[3] = (value & 0x00ff) as u8;

        self
    }
}

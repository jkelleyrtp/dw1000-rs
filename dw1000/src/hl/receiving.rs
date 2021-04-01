use crate::{mac, time::Instant, Error, Ready, Receiving, DW1000};
use byte::BytesExt as _;
use core::convert::TryInto;
use embedded_hal::{blocking::spi, digital::v2::OutputPin};
use fixed::traits::LossyInto;
use ieee802154::mac::FooterMode;

/// An incoming message
#[derive(Debug)]
pub struct Message<'l> {
    /// The time the message was received
    ///
    /// This time is based on the local system time, as defined in the SYS_TIME
    /// register.
    pub rx_time: Instant,

    /// The MAC frame
    pub frame: mac::Frame<'l>,
}

/// A struct representing the quality of the received message.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RxQuality {
    /// The confidence that there was Line Of Sight between the sender and the receiver.
    ///
    /// - 0 means it's very unlikely there was LOS.
    /// - 1 means it's very likely there was LOS.
    ///
    /// The number doesn't give a guarantee, but an indication.
    /// It is based on the APS006_Part-3-DW1000-Diagnostics-for-NLOS-Channels-v1.1 document.
    pub los_confidence_level: f32,
    /// The radio signal strength indicator in dBm.
    ///
    /// The value is an estimation that is quite accurate up to -85 dBm.
    /// Above -85 dBm, the estimation underestimates the actual value.
    pub rssi: f32,
}

impl<SPI, CS> DW1000<SPI, CS, Receiving>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Wait for receive operation to finish
    ///
    /// This method returns an `nb::Result` to indicate whether the transmission
    /// has finished, or whether it is still ongoing. You can use this to busily
    /// wait for the transmission to finish, for example using `nb`'s `block!`
    /// macro, or you can use it in tandem with [`DW1000::enable_rx_interrupts`]
    /// and the DW1000 IRQ output to wait in a more energy-efficient manner.
    ///
    /// Handling the DW1000's IRQ output line is out of the scope of this
    /// driver, but please note that if you're using the DWM1001 module or
    /// DWM1001-Dev board, that the `dwm1001` crate has explicit support for
    /// this.
    pub fn wait<'b>(&mut self, buffer: &'b mut [u8]) -> nb::Result<Message<'b>, Error<SPI, CS>> {
        // ATTENTION:
        // If you're changing anything about which SYS_STATUS flags are being
        // checked in this method, also make sure to update `enable_interrupts`.
        let sys_status = self
            .ll()
            .sys_status()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        // Is a frame ready?
        if sys_status.rxdfr() == 0b0 {
            // No frame ready. Check for errors.
            if sys_status.rxfce() == 0b1 {
                return Err(nb::Error::Other(Error::Fcs));
            }
            if sys_status.rxphe() == 0b1 {
                return Err(nb::Error::Other(Error::Phy));
            }
            if sys_status.rxrfsl() == 0b1 {
                return Err(nb::Error::Other(Error::ReedSolomon));
            }
            if sys_status.rxrfto() == 0b1 {
                return Err(nb::Error::Other(Error::FrameWaitTimeout));
            }
            if sys_status.rxovrr() == 0b1 {
                return Err(nb::Error::Other(Error::Overrun));
            }
            if sys_status.rxpto() == 0b1 {
                return Err(nb::Error::Other(Error::PreambleDetectionTimeout));
            }
            if sys_status.rxsfdto() == 0b1 {
                return Err(nb::Error::Other(Error::SfdTimeout));
            }
            if sys_status.affrej() == 0b1 {
                return Err(nb::Error::Other(Error::FrameFilteringRejection));
            }
            // Some error flags that sound like valid errors aren't checked here,
            // because experience has shown that they seem to occur spuriously
            // without preventing a good frame from being received. Those are:
            // - LDEERR: Leading Edge Detection Processing Error
            // - RXPREJ: Receiver Preamble Rejection

            // No errors detected. That must mean the frame is just not ready
            // yet.
            return Err(nb::Error::WouldBlock);
        }

        // Frame is ready. Continue.

        // Wait until LDE processing is done. Before this is finished, the RX
        // time stamp is not available.
        if sys_status.ldedone() == 0b0 {
            return Err(nb::Error::WouldBlock);
        }
        let rx_time = self
            .ll()
            .rx_time()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?
            .rx_stamp();

        // `rx_time` comes directly from the register, which should always
        // contain a 40-bit timestamp. Unless the hardware or its documentation
        // are buggy, the following should never panic.
        let rx_time = unsafe { Instant::new_unchecked(rx_time) };

        // Reset status bits. This is not strictly necessary, but it helps, if
        // you have to inspect SYS_STATUS manually during debugging.
        self.ll()
            .sys_status()
            .write(
                |w| {
                    w.rxprd(0b1) // Receiver Preamble Detected
                        .rxsfdd(0b1) // Receiver SFD Detected
                        .ldedone(0b1) // LDE Processing Done
                        .rxphd(0b1) // Receiver PHY Header Detected
                        .rxphe(0b1) // Receiver PHY Header Error
                        .rxdfr(0b1) // Receiver Data Frame Ready
                        .rxfcg(0b1) // Receiver FCS Good
                        .rxfce(0b1) // Receiver FCS Error
                        .rxrfsl(0b1) // Receiver Reed Solomon Frame Sync Loss
                        .rxrfto(0b1) // Receiver Frame Wait Timeout
                        .ldeerr(0b1) // Leading Edge Detection Processing Error
                        .rxovrr(0b1) // Receiver Overrun
                        .rxpto(0b1) // Preamble Detection Timeout
                        .rxsfdto(0b1) // Receiver SFD Timeout
                        .rxrscs(0b1) // Receiver Reed-Solomon Correction Status
                        .rxprej(0b1)
                }, // Receiver Preamble Rejection
            )
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        // Read received frame
        let rx_finfo = self
            .ll()
            .rx_finfo()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;
        let rx_buffer = self
            .ll()
            .rx_buffer()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        let len = rx_finfo.rxflen() as usize;

        if buffer.len() < len {
            return Err(nb::Error::Other(Error::BufferTooSmall {
                required_len: len,
            }));
        }

        buffer[..len].copy_from_slice(&rx_buffer.data()[..len]);

        let frame = buffer[..len]
            .read_with(&mut 0, FooterMode::None)
            .map_err(|error| nb::Error::Other(Error::Frame(error)))?;

        Ok(Message { rx_time, frame })
    }

    fn calculate_luep(&mut self) -> Result<f32, Error<SPI, CS>> {
        #[allow(unused_imports)]
        use micromath::F32Ext;

        let rx_time_register = self.ll().rx_time().read()?;
        let rx_fqual_register = self.ll().rx_fqual().read()?;
        let lde_cfg1_register = self.ll().lde_cfg1().read()?;

        let path_position: f32 =
            fixed::types::U10F6::from_le_bytes(rx_time_register.fp_index().to_le_bytes())
                .lossy_into();

        // Calculate a new low threshold by taking 0.6 times the reported noise threshold from the
        // diagnostics. This new threshold is shown in red in Figure 5. Get existing noise threshold as the
        // multiplication of STD_NOISE from Register 12:00 and NTM from Register 2E:0806.
        let noise_threshold: u16 = rx_fqual_register.std_noise() * lde_cfg1_register.ntm() as u16;
        let new_low_threshold = (noise_threshold as f32 * 0.6) as u16;
        // From the integer part of the first path position, pathPosition,
        // form an analysis window of 16 samples back tracked from that index.
        const WINDOW_SIZE: usize = 16;
        let window_start = path_position as u16 - WINDOW_SIZE as u16;

        let mut cir_buffer = [0u8; WINDOW_SIZE * 4 + 1];
        self.ll.cir(window_start * 4, &mut cir_buffer)?;
        let cir = &cir_buffer[1..];

        // To determine the number of peaks in the newly formed analysis window we take the difference of consecutive values.
        // We identify a peak when these differences change from positive to negative.

        // Calculate the amplitudes in the cir buffer
        let mut amplitudes = [0.0; WINDOW_SIZE];
        let mut peak_count = 0;
        for index in 0..WINDOW_SIZE {
            let real = u16::from_le_bytes(cir[index * 4..index * 4 + 2].try_into().unwrap()) as f32;
            let imag =
                u16::from_le_bytes(cir[index * 4 + 2..index * 4 + 4].try_into().unwrap()) as f32;

            amplitudes[index] = (real * real + imag * imag).sqrt();

            if index >= 2 && amplitudes[index - 1] > new_low_threshold as f32 {
                let previous_difference = amplitudes[index - 1] - amplitudes[index - 2];
                let current_difference = amplitudes[index] - amplitudes[index - 1];
                peak_count += (previous_difference.is_sign_positive()
                    && current_difference.is_sign_negative()) as u8;
            }
        }

        Ok(peak_count as f32 / (WINDOW_SIZE / 2) as f32)
    }

    fn calculate_prnlos(&mut self) -> Result<f32, Error<SPI, CS>> {
        #[allow(unused_imports)]
        use micromath::F32Ext;

        let rx_time_register = self.ll().rx_time().read()?;

        let path_position: f32 =
            fixed::types::U10F6::from_le_bytes(rx_time_register.fp_index().to_le_bytes())
                .lossy_into();

        let peak_path_index: f32 = self.ll().lde_ppindx().read()?.value() as f32;

        let idiff = (path_position - peak_path_index).abs();
        if idiff <= 3.3 {
            Ok(0.0)
        } else if idiff < 6.0 {
            Ok(0.39178 * idiff - 1.31719)
        } else {
            Ok(1.0)
        }
    }

    fn calculate_mc(&mut self) -> Result<f32, Error<SPI, CS>> {
        let rx_time_register = self.ll().rx_time().read()?;
        let rx_fqual_register = self.ll().rx_fqual().read()?;

        let fp_ampl1: u16 = rx_time_register.fp_ampl1();
        let fp_ampl2: u16 = rx_fqual_register.fp_ampl2();
        let fp_ampl3: u16 = rx_fqual_register.fp_ampl3();
        let peak_path_amplitude: u16 = self.ll().lde_ppampl().read()?.value();

        Ok(fp_ampl1.max(fp_ampl2).max(fp_ampl3) as f32 / peak_path_amplitude as f32)
    }

    fn calculate_rssi(&mut self) -> Result<f32, Error<SPI, CS>> {
        #[allow(unused_imports)]
        use micromath::F32Ext;

        let c = self.ll.rx_fqual().read()?.cir_pwr() as f32;
        let a = match self.state.used_config.pulse_repetition_frequency {
            crate::configs::PulseRepetitionFrequency::Mhz16 => 113.77,
            crate::configs::PulseRepetitionFrequency::Mhz64 => 121.74,
        };

        let data_rate = self.state.used_config.bitrate;
        let sfd_sequence = self.state.used_config.sfd_sequence;

        let rxpacc = self.ll.rx_finfo().read()?.rxpacc();
        let rxpacc_nosat = self.ll.rxpacc_nosat().read()?.value();

        let n = if rxpacc == rxpacc_nosat {
            rxpacc as f32 + sfd_sequence.get_rxpacc_adjustment(data_rate) as f32
        } else {
            rxpacc as f32
        };

        let rssi = 10.0 * ((c * (1 << 17) as f32) / (n * n)).log10() - a;

        if rssi.is_finite() {
            Ok(rssi)
        } else {
            Err(Error::BadRssiCalculation)
        }
    }

    /// Reads the quality of the received message.
    ///
    /// This must be called after the [`DW1000::wait`] function has successfully returned.
    pub fn read_rx_quality(&mut self) -> Result<RxQuality, Error<SPI, CS>> {
        let luep = self.calculate_luep()?;
        let prnlos = self.calculate_prnlos()?;
        let mc = self.calculate_mc()?;

        let los_confidence_level = if luep > 0.0 {
            0.0
        } else if prnlos == 0.0 || mc >= 0.9 {
            1.0
        } else {
            1.0 - prnlos
        };

        let rssi = self.calculate_rssi()?;

        Ok(RxQuality {
            los_confidence_level: los_confidence_level.clamp(0.0, 1.0),
            rssi,
        })
    }

    /// Gets the external sync values from the registers.
    ///
    /// The tuple contains (cycles_since_sync, nanos_until_tick, raw_timestamp).
    /// See the user manual at 6.1.3 to see how to calculate the actual time value.
    /// In the manual, the return values are named (N, T1, RX_RAWST)
    /// This is left to the user so the precision of the calculations are left to the user to decide.
    pub fn read_external_sync_time(&mut self) -> Result<(u32, u8, u64), Error<SPI, CS>> {
        let cycles_since_sync = self.ll().ec_rxtc().read()?.rx_ts_est();
        let nanos_until_tick = self.ll().ec_golp().read()?.offset_ext();
        let raw_timestamp = self.ll().rx_time().read()?.rx_rawst();

        Ok((cycles_since_sync, nanos_until_tick, raw_timestamp))
    }

    /// Finishes receiving and returns to the `Ready` state
    ///
    /// If the receive operation has finished, as indicated by `wait`, this is a
    /// no-op. If the receive operation is still ongoing, it will be aborted.
    pub fn finish_receiving(mut self) -> Result<DW1000<SPI, CS, Ready>, (Self, Error<SPI, CS>)> {
        if !self.state.finished {
            // Can't use `map_err` and `?` here, as the compiler will complain
            // about `self` moving into the closure.
            match self.force_idle() {
                Ok(()) => (),
                Err(error) => return Err((self, error)),
            }
        }

        Ok(DW1000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }
}

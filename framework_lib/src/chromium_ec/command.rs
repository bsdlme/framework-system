#[cfg(feature = "uefi")]
use core::prelude::rust_2021::derive;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::util;

use super::{CrosEc, CrosEcDriver, EcError, EcResult};

#[non_exhaustive]
#[derive(Debug, FromPrimitive)]
#[repr(u16)]
pub enum EcCommands {
    GetVersion = 0x02,
    GetBuildInfo = 0x04,
    /// Command to read data from EC memory map
    ReadMemMap = 0x07,
    GetCmdVersions = 0x08,
    FlashInfo = 0x10,
    /// Write section of EC flash
    FlashRead = 0x11,
    /// Write section of EC flash
    FlashWrite = 0x12,
    /// Erase section of EC flash
    FlashErase = 0x13,
    FlashProtect = 0x15,
    PwmGetKeyboardBacklight = 0x0022,
    PwmSetKeyboardBacklight = 0x0023,
    GpioGet = 0x93,
    I2cPassthrough = 0x9e,
    ConsoleSnapshot = 0x97,
    ConsoleRead = 0x98,
    /// List the features supported by the firmware
    GetFeatures = 0x0D,
    /// Force reboot, causes host reboot as well
    Reboot = 0xD1,
    /// Control EC boot
    RebootEc = 0xD2,
    /// Get information about PD controller power
    UsbPdPowerInfo = 0x103,

    // Framework specific commands
    /// Configure the behavior of the flash notify
    FlashNotified = 0x3E01,
    /// Change charge limit
    ChargeLimitControl = 0x3E03,
    /// Get/Set Fingerprint LED brightness
    FpLedLevelControl = 0x3E0E,
    /// Get information about the current chassis open/close status
    ChassisOpenCheck = 0x3E0F,
    /// Get information about historical chassis open/close (intrusion) information
    ChassisIntrusion = 0x3E09,

    /// Not used by this library
    AcpiNotify = 0xE10,

    /// Get information about PD controller version
    ReadPdVersion = 0x3E11,

    /// Not used by this library
    StandaloneMode = 0x3E13,
    /// Get information about current state of privacy switches
    PriavcySwitchesCheckMode = 0x3E14,
    /// Not used by this library
    ChassisCounter = 0x3E15,
    /// On Framework 16, check the status of the input module deck
    CheckDeckState = 0x3E16,
    /// Not used by this library
    GetSimpleVersion = 0x3E17,
    /// GetActiveChargePdChip
    GetActiveChargePdChip = 0x3E18,

    /// Set UEFI App mode
    UefiAppMode = 0x3E19,
    /// Get UEFI APP Button status
    UefiAppBtnStatus = 0x3E1A,
    /// Get expansion bay status
    ExpansionBayStatus = 0x3E1B,
    /// Get hardware diagnostics
    GetHwDiag = 0x3E1C,
}

pub trait EcRequest<R> {
    fn command_id() -> EcCommands;
    // Can optionally override this
    fn command_version() -> u8 {
        0
    }
}

impl<T: EcRequest<R>, R> EcRequestRaw<R> for T {
    fn command_id_u16() -> u16 {
        Self::command_id() as u16
    }
    fn command_version() -> u8 {
        Self::command_version()
    }
}

pub trait EcRequestRaw<R> {
    fn command_id_u16() -> u16;
    fn command_version() -> u8;

    fn format_request(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe { util::any_as_u8_slice(self) }
    }

    fn send_command_vec(&self, ec: &CrosEc) -> EcResult<Vec<u8>>
    where
        Self: Sized,
    {
        self.send_command_vec_extra(ec, &[])
    }

    fn send_command_vec_extra(&self, ec: &CrosEc, extra_data: &[u8]) -> EcResult<Vec<u8>>
    where
        Self: Sized,
    {
        let params = self.format_request();
        let request = if extra_data.is_empty() {
            params.to_vec()
        } else {
            let mut buffer: Vec<u8> = vec![0; params.len() + extra_data.len()];
            buffer[..params.len()].copy_from_slice(params);
            buffer[params.len()..].copy_from_slice(extra_data);
            buffer
        };
        let response =
            ec.send_command(Self::command_id_u16(), Self::command_version(), &request)?;
        trace!(
            "send_command<{:X?}>",
            <EcCommands as FromPrimitive>::from_u16(Self::command_id_u16())
        );
        trace!("  Request:  {:?}", request);
        trace!("  Response: {:?}", response);
        Ok(response)
    }

    fn send_command(&self, ec: &CrosEc) -> EcResult<R>
    where
        Self: Sized,
    {
        self.send_command_extra(ec, &[])
    }

    // Same as send_command but with extra data packed after the defined struct
    fn send_command_extra(&self, ec: &CrosEc, extra_data: &[u8]) -> EcResult<R>
    where
        Self: Sized,
    {
        let response = self.send_command_vec_extra(ec, extra_data)?;
        // TODO: The Windows driver seems to return 20 more bytes than expected
        #[cfg(feature = "win_driver")]
        let expected = response.len() != std::mem::size_of::<R>() + 20;
        #[cfg(not(feature = "win_driver"))]
        let expected = response.len() != std::mem::size_of::<R>();
        if expected {
            return Err(EcError::DeviceError(format!(
                "Returned data size ({}) is not the expted size: {}",
                response.len(),
                std::mem::size_of::<R>()
            )));
        }
        let val: R = unsafe { std::ptr::read(response.as_ptr() as *const _) };
        Ok(val)
    }
}

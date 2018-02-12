mod raw;

use std::error;
use std::fmt;
use std::result;
use std::mem;
use std::ffi::CStr;

pub use raw::I2C_TRANSFER_OPTIONS_START_BIT;
pub use raw::I2C_TRANSFER_OPTIONS_STOP_BIT;
pub use raw::I2C_TRANSFER_OPTIONS_BREAK_ON_NACK;
pub use raw::I2C_TRANSFER_OPTIONS_NACK_LAST_BYTE;
pub use raw::I2C_TRANSFER_OPTIONS_FAST_TRANSFER_BYTES;
pub use raw::I2C_TRANSFER_OPTIONS_FAST_TRANSFER_BITS;
pub use raw::I2C_TRANSFER_OPTIONS_FAST_TRANSFER;
pub use raw::I2C_TRANSFER_OPTIONS_NO_ADDRESS;

pub use raw::I2C_CLOCKRATE as ClockRate;

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    InvalidHandle,
    DeviceNotFound,
    DeviceNotOpened,
    IoError,
    InsufficientResources,
    InvalidParameter,
    InvalidBaudRate,
    DeviceNotOpenedForErase,
    DeviceNotOpenedForWrite,
    FailedToWriteDevice,
    EepromReadFailed,
    EepromWriteFailed,
    EepromEraseFailed,
    EepromNotPresent,
    EepromNotProgrammed,
    InvalidArgs,
    NotSupported,
    OtherError,
}

impl Error {
    fn new(status: raw::FT_STATUS) -> Error {
        match status {
            1 => Error::InvalidHandle,
            2 => Error::DeviceNotFound,
            3 => Error::DeviceNotOpened,
            4 => Error::IoError,
            5 => Error::InsufficientResources,
            6 => Error::InvalidParameter,
            7 => Error::InvalidBaudRate,
            8 => Error::DeviceNotOpenedForErase,
            9 => Error::DeviceNotOpenedForWrite,
            10 => Error::FailedToWriteDevice,
            11 => Error::EepromReadFailed,
            12 => Error::EepromWriteFailed,
            13 => Error::EepromEraseFailed,
            14 => Error::EepromNotPresent,
            15 => Error::EepromNotProgrammed,
            16 => Error::InvalidArgs,
            17 => Error::NotSupported,
            18 => Error::OtherError,
            _ => unreachable!(),
        }
    }
}

impl Error {
    fn as_str(&self) -> &'static str {
        match *self {
            Error::InvalidHandle => "invalid handle",
            Error::DeviceNotFound => "device not found",
            Error::DeviceNotOpened => "device not opened",
            Error::IoError => "io error",
            Error::InsufficientResources => "insufficient resources",
            Error::InvalidParameter => "invalid parameter",
            Error::InvalidBaudRate => "invalid baud rate",
            Error::DeviceNotOpenedForErase => "device not opened for erase",
            Error::DeviceNotOpenedForWrite => "device not opened for write",
            Error::FailedToWriteDevice => "failed to write device",
            Error::EepromReadFailed => "EEPROM read failed",
            Error::EepromWriteFailed => "EEPROM write failed",
            Error::EepromEraseFailed => "EEPROM erase failed",
            Error::EepromNotPresent => "EEPROM not present",
            Error::EepromNotProgrammed => "EEPROM not programmed",
            Error::InvalidArgs => "invalid args",
            Error::NotSupported => "not supported",
            Error::OtherError => "other error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        self.as_str().fmt(f)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

fn check(status: raw::FT_STATUS) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(Error::new(status))
    }
}

pub fn get_num_channels() -> Result<usize> {
    unsafe {
        let mut num: u32 = mem::uninitialized();
        check(raw::I2C_GetNumChannels(&mut num))?;
        Ok(num as usize)
    }
}

#[derive(Debug)]
pub struct ChannelInfo {
    pub serial_number: String,
    pub description: String,
}

impl ChannelInfo {
    fn new(node: raw::FT_DEVICE_LIST_INFO_NODE) -> ChannelInfo {
        unsafe {
            ChannelInfo {
                serial_number: CStr::from_ptr(node.SerialNumber.as_ptr())
                    .to_string_lossy()
                    .to_string(),
                description: CStr::from_ptr(node.Description.as_ptr())
                    .to_string_lossy()
                    .to_string(),
            }
        }
    }
}

pub fn get_channel_info(index: usize) -> Result<ChannelInfo> {
    let n = get_num_channels()?;
    if index >= n {
        return Err(Error::InvalidArgs);
    }
    unsafe {
        let mut node = mem::uninitialized();
        check(raw::I2C_GetChannelInfo(index as u32, &mut node))?;
        Ok(ChannelInfo::new(node))
    }
}

pub struct ChannelHandle {
    raw: raw::FT_HANDLE,
}

impl ChannelHandle {
    pub fn open(
        index: usize,
        clock_rate: ClockRate,
        latency_timer: u8,
        options: u32,
    ) -> Result<ChannelHandle> {
        let n = get_num_channels()?;
        if index >= n {
            return Err(Error::InvalidArgs);
        }
        let handle = unsafe {
            let mut handle = mem::uninitialized();
            check(raw::I2C_OpenChannel(index as u32, &mut handle))?;
            ChannelHandle { raw: handle }
        };
        unsafe {
            check(raw::I2C_InitChannel(
                handle.raw,
                &mut raw::ChannelConfig {
                    ClockRate: clock_rate,
                    LatencyTimer: latency_timer,
                    Options: options,
                },
            ))?;
        }
        Ok(handle)
    }

    pub fn read(&self, device: u32, options: u32, buf: &mut [u8]) -> Result<usize> {
        let n = buf.len();
        unsafe {
            let mut size = mem::uninitialized();
            check(raw::I2C_DeviceRead(
                self.raw,
                device,
                n as u32,
                buf.as_mut_ptr(),
                &mut size,
                options,
            ))?;
            Ok(size as usize)
        }
    }

    pub fn write(&mut self, device: u32, options: u32, buf: &[u8]) -> Result<usize> {
        let n = buf.len();
        let buf = &mut Vec::from(buf);
        unsafe {
            let mut size = mem::uninitialized();
            check(raw::I2C_DeviceWrite(
                self.raw,
                device,
                n as u32,
                buf.as_mut_ptr(),
                &mut size,
                options,
            ))?;
            Ok(size as usize)
        }
    }
}

impl Drop for ChannelHandle {
    fn drop(&mut self) {
        unsafe {
            check(raw::I2C_CloseChannel(self.raw)).unwrap(); // TODO: error handling
        }
    }
}

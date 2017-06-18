//! * Driver for the LPS25H Pressure sensor
//! See http://www.st.com/en/mems-and-sensors/lps25h.html

use i2cdev::core::I2CDevice;
use byteorder::{ByteOrder, LittleEndian};

pub const REG_RES_CONF: u8 = 0x10;
pub const REG_CTRL_REG_1: u8 = 0x20;
pub const REG_CTRL_REG_2: u8 = 0x21;
pub const REG_STATUS_REG: u8 = 0x27;
pub const REG_PRESS_OUT_XL: u8 = 0x28;
pub const REG_PRESS_OUT_L: u8 = 0x29;
pub const REG_PRESS_OUT_H: u8 = 0x2a;
pub const REG_TEMP_OUT_L: u8 = 0x2b;
pub const REG_TEMP_OUT_H: u8 = 0x2c;
pub const REG_FIFO_CTRL: u8 = 0x2e;

pub struct Lps25h<T: I2CDevice + Sized> {
    i2cdev: T,
}

impl<T> Lps25h<T>
    where T: I2CDevice + Sized
{
    /// Create a new pressure sensor handle for the given path/addr.
    /// Init sequence from https://github.com/RPi-Distro/RTIMULib
    pub fn new(mut i2cdev: T) -> Result<Lps25h<T>, T::Error> {
        i2cdev.smbus_write_byte_data(REG_CTRL_REG_1, 0xc4)?;
        i2cdev.smbus_write_byte_data(REG_RES_CONF, 0x05)?;
        i2cdev.smbus_write_byte_data(REG_FIFO_CTRL, 0xc0)?;
        i2cdev.smbus_write_byte_data(REG_CTRL_REG_2, 0x40)?;

        Ok(Lps25h { i2cdev: i2cdev })
    }

    /// Obtain the status bitfield from the chip.
    pub fn status(&mut self) -> Result<u8, T::Error> {
        self.i2cdev.smbus_read_byte_data(REG_STATUS_REG)
    }

    /// Obtain the temperature reading from the chip.
    /// T(°C) = 42.5 + (TEMP_OUT / 480)
    pub fn get_temp(&mut self) -> Result<i16, T::Error> {
        let mut buf = [0u8; 2];
        buf[0] = self.i2cdev.smbus_read_byte_data(REG_TEMP_OUT_L)?;
        buf[1] = self.i2cdev.smbus_read_byte_data(REG_TEMP_OUT_H)?;
        Ok(LittleEndian::read_i16(&buf))
    }

    /// Obtain the temperature reading from the chip in deg C.
    pub fn get_temp_celcius(&mut self) -> Result<f64, T::Error> {
        self.get_temp().and_then(|c| Ok((c as f64 / 480.0) + 42.5))
    }

    /// Obtain the pressure reading from the chip.
    /// Pout(hPa) = PRESS_OUT / 4096
    pub fn get_pressure(&mut self) -> Result<u32, T::Error> {
        let mut buf = [0u8; 4];
        buf[0] = self.i2cdev.smbus_read_byte_data(REG_PRESS_OUT_XL)?;
        buf[1] = self.i2cdev.smbus_read_byte_data(REG_PRESS_OUT_L)?;
        buf[2] = self.i2cdev.smbus_read_byte_data(REG_PRESS_OUT_H)?;
        Ok(LittleEndian::read_u32(&buf))
    }

    /// Obtain the pressure reading from the chip in hPa.
    pub fn get_pressure_hpa(&mut self) -> Result<f64, T::Error> {
        self.get_pressure().and_then(|c| Ok(c as f64 / 4096.0))
    }
}

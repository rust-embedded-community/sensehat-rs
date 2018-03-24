//! * Driver for the HTS221 humidity sensor
//! See `http://www.st.com/content/st_com/en/products/mems-and-sensors/humidity-sensors/hts221.html`

use i2cdev::core::I2CDevice;
use byteorder::{ByteOrder, LittleEndian};

pub const REG_AV_CONF: u8 = 0x10;
pub const REG_CTRL1: u8 = 0x20;
pub const REG_STATUS: u8 = 0x27;
pub const REG_HUMIDITY_OUT_L: u8 = 0x28;
pub const REG_HUMIDITY_OUT_H: u8 = 0x29;
pub const REG_TEMP_OUT_L: u8 = 0x2a;
pub const REG_TEMP_OUT_H: u8 = 0x2b;
pub const REG_H0_H_2: u8 = 0x30;
pub const REG_H1_H_2: u8 = 0x31;
pub const REG_T0_C_8: u8 = 0x32;
pub const REG_T1_C_8: u8 = 0x33;
pub const REG_T1_T0: u8 = 0x35;
pub const REG_H0_T0_OUT: u8 = 0x36;
pub const REG_H1_T0_OUT: u8 = 0x3a;
pub const REG_T0_OUT: u8 = 0x3c;
pub const REG_T1_OUT: u8 = 0x3e;

pub struct Hts221<T: I2CDevice + Sized> {
    i2cdev: T,
    temp_m: f64,
    temp_c: f64,
    hum_m: f64,
    hum_c: f64,
}

impl<T> Hts221<T>
where
    T: I2CDevice + Sized,
{
    /// Create a new pressure sensor handle for the given path/addr.
    /// Init sequence from https://github.com/RPi-Distro/RTIMULib
    pub fn new(mut i2cdev: T) -> Result<Hts221<T>, T::Error> {
        // Init

        i2cdev.smbus_write_byte_data(REG_CTRL1, 0x87)?;
        i2cdev.smbus_write_byte_data(REG_AV_CONF, 0x1b)?;

        // Get cal
        let mut buf = [0u8; 2];
        buf[0] = i2cdev.smbus_read_byte_data(REG_T0_C_8)?;
        buf[1] = i2cdev.smbus_read_byte_data(REG_T1_T0)? & 0x03;
        let t0 = f64::from(LittleEndian::read_i16(&buf)) / 8.0;
        buf[0] = i2cdev.smbus_read_byte_data(REG_T1_C_8)?;
        buf[1] = (i2cdev.smbus_read_byte_data(REG_T1_T0)? & 0x0C) >> 2;
        let t1 = f64::from(LittleEndian::read_i16(&buf)) / 8.0;

        buf[0] = i2cdev.smbus_read_byte_data(REG_T0_OUT)?;
        buf[1] = i2cdev.smbus_read_byte_data(REG_T0_OUT + 1)?;
        let t0_out = f64::from(LittleEndian::read_i16(&buf));

        buf[0] = i2cdev.smbus_read_byte_data(REG_T1_OUT)?;
        buf[1] = i2cdev.smbus_read_byte_data(REG_T1_OUT + 1)?;
        let t1_out = f64::from(LittleEndian::read_i16(&buf));

        buf[0] = i2cdev.smbus_read_byte_data(REG_H0_H_2)?;
        let h0 = f64::from(buf[0]) / 2.0;

        buf[0] = i2cdev.smbus_read_byte_data(REG_H1_H_2)?;
        let h1 = f64::from(buf[0]) / 2.0;

        buf[0] = i2cdev.smbus_read_byte_data(REG_H0_T0_OUT)?;
        buf[1] = i2cdev.smbus_read_byte_data(REG_H0_T0_OUT + 1)?;
        let h0_t0_out = f64::from(LittleEndian::read_i16(&buf));

        buf[0] = i2cdev.smbus_read_byte_data(REG_H1_T0_OUT)?;
        buf[1] = i2cdev.smbus_read_byte_data(REG_H1_T0_OUT + 1)?;
        let h1_t0_out = f64::from(LittleEndian::read_i16(&buf));

        let temp_m = (t1 - t0) / (t1_out - t0_out);
        let temp_c = t0 - (temp_m * t0_out);
        let hum_m = (h1 - h0) / (h1_t0_out - h0_t0_out);
        let hum_c = h0 - (hum_m * h0_t0_out);

        Ok(Hts221 {
            i2cdev,
            temp_m,
            temp_c,
            hum_m,
            hum_c,
        })
    }

    /// Obtain the status bitfield from the chip.
    pub fn status(&mut self) -> Result<u8, T::Error> {
        self.i2cdev.smbus_read_byte_data(REG_STATUS)
    }

    pub fn get_relative_humidity(&mut self) -> Result<i16, T::Error> {
        let mut buf = [0u8; 2];
        buf[0] = self.i2cdev.smbus_read_byte_data(REG_HUMIDITY_OUT_L)?;
        buf[1] = self.i2cdev.smbus_read_byte_data(REG_HUMIDITY_OUT_H)?;
        Ok(LittleEndian::read_i16(&buf))
    }

    pub fn get_relative_humidity_percent(&mut self) -> Result<f64, T::Error> {
        self.get_relative_humidity()
            .and_then(|c| Ok((f64::from(c) * self.hum_m) + self.hum_c))
    }

    pub fn get_temperature(&mut self) -> Result<i16, T::Error> {
        let mut buf = [0u8; 2];
        buf[0] = self.i2cdev.smbus_read_byte_data(REG_TEMP_OUT_L)?;
        buf[1] = self.i2cdev.smbus_read_byte_data(REG_TEMP_OUT_H)?;
        Ok(LittleEndian::read_i16(&buf))
    }

    pub fn get_temperature_celcius(&mut self) -> Result<f64, T::Error> {
        self.get_temperature()
            .and_then(|c| Ok((f64::from(c) * self.temp_m) + self.temp_c))
    }
}

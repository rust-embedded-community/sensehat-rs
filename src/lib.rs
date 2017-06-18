extern crate byteorder;
extern crate i2cdev;
extern crate measurements;

pub use measurements::Temperature;
pub use measurements::Pressure;

use i2cdev::core::I2CDevice;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use byteorder::{ByteOrder, LittleEndian};

use std::fmt;

mod ht221;
mod lps25h;

/// Represents a relative humidity reading from the humidity sensor
pub struct RelativeHumidity {
    value: f64,
}

/// Represents the SenseHat itself
pub struct SenseHat {
    // LPS25H pressure sensor
    pressure_chip: lps25h::Lps25h<LinuxI2CDevice>,
    // HT221 humidity sensor
    humidity_dev: LinuxI2CDevice,
    temp_m: f64,
    temp_c: f64,
    hum_m: f64,
    hum_c: f64,
}

/// Errors that this crate can return
#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    I2CError(LinuxI2CError),
}

/// A shortcut for Results that can return `T` or `SenseHatError`
pub type SenseHatResult<T> = Result<T, SenseHatError>;

impl SenseHat {
    /// Try and create a new SenseHat object.
    ///
    /// Will open the relevant I2C devices and then attempt to initialise the
    /// chips on the Sense Hat.
    pub fn new() -> SenseHatResult<SenseHat> {
        let mut hat = SenseHat {
            humidity_dev: LinuxI2CDevice::new("/dev/i2c-1", 0x5f)?,
            pressure_chip: lps25h::Lps25h::new(LinuxI2CDevice::new("/dev/i2c-1", 0x5c)?)?,
            temp_m: 0.0,
            temp_c: 0.0,
            hum_m: 0.0,
            hum_c: 0.0,
        };

        hat.init_humidity()?;

        Ok(hat)
    }

    /// Init sequence from https://github.com/RPi-Distro/RTIMULib
    fn init_humidity(&mut self) -> SenseHatResult<()> {
        // Init
        self.humidity_dev.smbus_write_byte_data(ht221::REG_CTRL1, 0x87)?;
        self.humidity_dev.smbus_write_byte_data(ht221::REG_AV_CONF, 0x1b)?;

        // Get cal
        let mut buf = [0u8; 2];
        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T0_C_8)?;
        buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T1_T0)? & 0x03;
        let t0 = (LittleEndian::read_i16(&buf) as f64) / 8.0;
        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T1_C_8)?;
        buf[1] = (self.humidity_dev.smbus_read_byte_data(ht221::REG_T1_T0)? & 0x0C) >> 2;
        let t1 = (LittleEndian::read_i16(&buf) as f64) / 8.0;

        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T0_OUT)?;
        buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T0_OUT + 1)?;
        let t0_out = LittleEndian::read_i16(&buf) as f64;

        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T1_OUT)?;
        buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_T1_OUT + 1)?;
        let t1_out = LittleEndian::read_i16(&buf) as f64;

        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_H0_H_2)?;
        let h0 = (buf[0] as f64) / 2.0;

        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_H1_H_2)?;
        let h1 = (buf[0] as f64) / 2.0;

        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_H0_T0_OUT)?;
        buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_H0_T0_OUT + 1)?;
        let h0_t0_out = LittleEndian::read_i16(&buf) as f64;

        buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_H1_T0_OUT)?;
        buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_H1_T0_OUT + 1)?;
        let h1_t0_out = LittleEndian::read_i16(&buf) as f64;

        self.temp_m = (t1 - t0) / (t1_out - t0_out);
        self.temp_c = t0 - (self.temp_m * t0_out);
        self.hum_m = (h1 - h0) / (h1_t0_out - h0_t0_out);
        self.hum_c = h0 - (self.hum_m * h0_t0_out);

        Ok(())
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        let status = self.pressure_chip.status()?;
        if (status & 1) != 0 {
            let celcius = ((self.pressure_chip.read_temp()? as f64) / 480.0) + 42.5;
            Ok(Temperature::from_celsius(celcius))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Pressure value from the barometer
    pub fn get_pressure(&mut self) -> SenseHatResult<Pressure> {
        let status = self.pressure_chip.status()?;
        if (status & 2) != 0 {
            let hectopascals = (self.pressure_chip.read_pressure()? as f64) / 4096.0;
            Ok(Pressure::from_hectopascals(hectopascals))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Temperature reading from the humidity sensor. It's more
    /// accurate than the barometer (+/- 0.5 degrees C), but over a smaller
    /// range.
    pub fn get_temperature_from_humidity(&mut self) -> SenseHatResult<Temperature> {
        let status = self.humidity_dev.smbus_read_byte_data(ht221::REG_STATUS)?;
        if (status & 1) != 0 {
            let mut buf = [0u8; 2];
            buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_TEMP_OUT_L)?;
            buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_TEMP_OUT_H)?;
            let celcius = ((LittleEndian::read_i16(&buf) as f64) * self.temp_m) + self.temp_c;
            Ok(Temperature::from_celsius(celcius))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a RelativeHumidity value in percent between 0 and 100
    pub fn get_humidity(&mut self) -> SenseHatResult<RelativeHumidity> {
        let status = self.humidity_dev.smbus_read_byte_data(ht221::REG_STATUS)?;
        if (status & 2) != 0 {
            let mut buf = [0u8; 2];
            buf[0] = self.humidity_dev.smbus_read_byte_data(ht221::REG_HUMIDITY_OUT_L)?;
            buf[1] = self.humidity_dev.smbus_read_byte_data(ht221::REG_HUMIDITY_OUT_H)?;
            let percent = ((LittleEndian::read_i16(&buf) as f64) * self.hum_m) + self.hum_c;
            Ok(RelativeHumidity::from_percent(percent))
        } else {
            Err(SenseHatError::NotReady)
        }
    }
}

impl From<LinuxI2CError> for SenseHatError {
    fn from(err: LinuxI2CError) -> SenseHatError {
        SenseHatError::I2CError(err)
    }
}

impl RelativeHumidity {
    pub fn from_percent(pc: f64) -> RelativeHumidity {
        RelativeHumidity { value: pc }
    }

    pub fn as_percent(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for RelativeHumidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}%", self.as_percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn pressure_test() {
        let p = Pressure::from_hectopascals(1000.0);
        assert_eq!(p.as_bars(), 1.0);
        assert_eq!(p.as_psi(), 14.5038);
    }
}

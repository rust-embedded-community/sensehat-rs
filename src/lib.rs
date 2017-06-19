extern crate byteorder;
extern crate i2cdev;
extern crate measurements;

pub use measurements::Temperature;
pub use measurements::Pressure;

use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use std::fmt;

mod hts221;
mod lps25h;
mod lsm9ds1;

/// Represents a relative humidity reading from the humidity sensor
pub struct RelativeHumidity {
    value: f64,
}

/// Represents the SenseHat itself
pub struct SenseHat<'a> {
    // LPS25H pressure sensor
    pressure_chip: lps25h::Lps25h<LinuxI2CDevice>,
    // HTS221 humidity sensor
    humidity_chip: hts221::Hts221<LinuxI2CDevice>,
    // LSM9DS1 IMU device
    accelerometer_chip: lsm9ds1::Lsm9ds1<'a>,
}

/// Errors that this crate can return
#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    I2CError(LinuxI2CError),
    LSM9DS1Error(lsm9ds1::Error),
}

/// A shortcut for Results that can return `T` or `SenseHatError`
pub type SenseHatResult<T> = Result<T, SenseHatError>;

impl<'a> SenseHat<'a> {
    /// Try and create a new SenseHat object.
    ///
    /// Will open the relevant I2C devices and then attempt to initialise the
    /// chips on the Sense Hat.
    pub fn new() -> SenseHatResult<SenseHat<'a>> {
        Ok(SenseHat {
               humidity_chip: hts221::Hts221::new(LinuxI2CDevice::new("/dev/i2c-1", 0x5f)?)?,
               pressure_chip: lps25h::Lps25h::new(LinuxI2CDevice::new("/dev/i2c-1", 0x5c)?)?,
               accelerometer_chip: lsm9ds1::Lsm9ds1::new()?,
           })
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        let status = self.pressure_chip.status()?;
        if (status & 1) != 0 {
            Ok(Temperature::from_celsius(self.pressure_chip.get_temp_celcius()?))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Pressure value from the barometer
    pub fn get_pressure(&mut self) -> SenseHatResult<Pressure> {
        let status = self.pressure_chip.status()?;
        if (status & 2) != 0 {
            Ok(Pressure::from_hectopascals(self.pressure_chip.get_pressure_hpa()?))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Temperature reading from the humidity sensor. It's more
    /// accurate than the barometer (+/- 0.5 degrees C), but over a smaller
    /// range.
    pub fn get_temperature_from_humidity(&mut self) -> SenseHatResult<Temperature> {
        let status = self.humidity_chip.status()?;
        if (status & 1) != 0 {
            let celcius = self.humidity_chip.get_temperature_celcius()?;
            Ok(Temperature::from_celsius(celcius))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a RelativeHumidity value in percent between 0 and 100
    pub fn get_humidity(&mut self) -> SenseHatResult<RelativeHumidity> {
        let status = self.humidity_chip.status()?;
        if (status & 2) != 0 {
            let percent = self.humidity_chip.get_relative_humidity_percent()?;
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

impl From<lsm9ds1::Error> for SenseHatError {
    fn from(err: lsm9ds1::Error) -> SenseHatError {
        SenseHatError::LSM9DS1Error(err)
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

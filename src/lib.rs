extern crate measurements;
extern crate i2cdev;

pub use measurements::Temperature;

use i2cdev::core::I2CDevice;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::fmt;

pub struct Pressure {
    value: f64,
}

pub struct RelativeHumidity {
    value: f64,
}

pub struct SenseHat {
    pressure_dev: LinuxI2CDevice,
    humidity_dev: LinuxI2CDevice,
}

#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    I2CError(LinuxI2CError),
}

pub type SenseHatResult<T> = Result<T, SenseHatError>;

// Registers for the LPS25H pressure sensor
// const LPS25H_REF_P_XL: u8 = 0x08;
// const LPS25H_REF_P_XH: u8 = 0x09;
const LPS25H_RES_CONF: u8 = 0x10;
const LPS25H_CTRL_REG_1: u8 = 0x20;
const LPS25H_CTRL_REG_2: u8 = 0x21;
// const LPS25H_CTRL_REG_3: u8 = 0x22;
// const LPS25H_CTRL_REG_4: u8 = 0x23;
// const LPS25H_INT_CFG: u8 = 0x24;
// const LPS25H_INT_SOURCE: u8 = 0x25;
const LPS25H_STATUS_REG: u8 = 0x27;
const LPS25H_PRESS_OUT_XL: u8 = 0x28;
const LPS25H_PRESS_OUT_L: u8 = 0x29;
const LPS25H_PRESS_OUT_H: u8 = 0x2a;
const LPS25H_TEMP_OUT_L: u8 = 0x2b;
const LPS25H_TEMP_OUT_H: u8 = 0x2c;
const LPS25H_FIFO_CTRL: u8 = 0x2e;
// const LPS25H_FIFO_STATUS: u8 = 0x2f;
// const LPS25H_THS_P_L: u8 = 0x30;
// const LPS25H_THS_P_H: u8 = 0x31;
// const LPS25H_RPDS_L: u8 = 0x39;
// const LPS25H_RPDS_H: u8 = 0x3a;

fn init_pressure(pressure: &mut LinuxI2CDevice) -> SenseHatResult<()> {
    // Init sequence from https://github.com/RPi-Distro/RTIMULib
    pressure.smbus_write_byte_data(LPS25H_CTRL_REG_1, 0xc4)?;
    pressure.smbus_write_byte_data(LPS25H_RES_CONF, 0x05)?;
    pressure.smbus_write_byte_data(LPS25H_FIFO_CTRL, 0xc0)?;
    pressure.smbus_write_byte_data(LPS25H_CTRL_REG_2, 0x40)?;
    Ok(())
}

fn init_humidity(hum: &mut LinuxI2CDevice) -> SenseHatResult<()> {
    // This is a bit more complicated...
    Ok(())
}

impl SenseHat {
    /// Try and create a new SenseHat object.
    ///
    /// Will open the relevant I2C devices and then attempt to initialise the
    /// chips on the Sense Hat.
    pub fn new() -> SenseHatResult<SenseHat> {
        let mut pressure = LinuxI2CDevice::new("/dev/i2c-1", 0x5c)?;
        let _ = init_pressure(&mut pressure)?;
        let mut humidity = LinuxI2CDevice::new("/dev/i2c-1", 0x5f)?;
        let _ = init_humidity(&mut humidity)?;
        Ok(SenseHat {
            pressure_dev: pressure,
            humidity_dev: humidity,
        })
    }

    /// Returns a Temperature reading from the humidity sensor. It's more
    /// accurate than the barometer (+/- 0.5 degrees C), but over a smaller
    /// range.
    pub fn get_temperature_from_humidity(&mut self) -> SenseHatResult<Temperature> {
        Ok(Temperature::from_celsius(36.7))
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        let status = self.pressure_dev.smbus_read_byte_data(LPS25H_STATUS_REG)?;
        if (status & 1) != 0 {
            let raw1 = self.pressure_dev.smbus_read_byte_data(LPS25H_TEMP_OUT_L)?;
            let raw2 = self.pressure_dev.smbus_read_byte_data(LPS25H_TEMP_OUT_H)?;
            let raw_total = (((raw2 as u16) << 8) + (raw1 as u16)) as i16;
            let celcius = ((raw_total as f64) / 480.0) + 42.5;
            Ok(Temperature::from_celsius(celcius))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Pressure value from the barometer
    pub fn get_pressure(&mut self) -> SenseHatResult<Pressure> {
        let status = self.pressure_dev.smbus_read_byte_data(LPS25H_STATUS_REG)?;
        if (status & 2) != 0 {
            let raw1 = self.pressure_dev.smbus_read_byte_data(LPS25H_PRESS_OUT_XL)?;
            let raw2 = self.pressure_dev.smbus_read_byte_data(LPS25H_PRESS_OUT_L)?;
            let raw3 = self.pressure_dev.smbus_read_byte_data(LPS25H_PRESS_OUT_H)?;
            let raw_total: u32 = ((raw3 as u32) << 16) + ((raw2 as u32) << 8) + (raw1 as u32);
            let hectopascals = (raw_total as f64) / 4096.0;
            Ok(Pressure::from_hectopascals(hectopascals))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a RelativeHumidity value in percent between 0 and 100
    pub fn get_humidity(&mut self) -> SenseHatResult<RelativeHumidity> {
        return Ok(RelativeHumidity::from_percent(50.0));
    }
}

impl From<LinuxI2CError> for SenseHatError {
    fn from(err: LinuxI2CError) -> SenseHatError {
        SenseHatError::I2CError(err)
    }
}

impl Pressure {
    /// hectopascals is the same as a millibar
    pub fn from_hectopascals(hectopascals: f64) -> Pressure {
        Pressure { value: hectopascals }
    }

    pub fn as_hectopascals(&self) -> f64 {
        self.value
    }

    pub fn as_millibars(&self) -> f64 {
        self.value
    }

    pub fn as_bar(&self) -> f64 {
        self.value / 1000.0
    }

    pub fn as_pascals(&self) -> f64 {
        self.value * 100.0
    }

    pub fn as_kilopascals(&self) -> f64 {
        self.value / 10.0
    }

    pub fn as_psi(&self) -> f64 {
        self.as_bar() * 14.5038
    }

    pub fn as_atmospheres(&self) -> f64 {
        self.as_bar() / 1.01325
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

impl fmt::Display for Pressure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.0} mbar", self.as_millibars())
    }
}

impl fmt::Display for RelativeHumidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%", self.as_percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn pressure_test() {
        let p = Pressure::from_hectopascals(1000.0);
        assert_eq!(p.as_bar(), 1.0);
        assert_eq!(p.as_psi(), 14.5038);
    }
}

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
    // LPS25H pressure sensor
    pressure_dev: LinuxI2CDevice,
    // HT221 humidity sensor
    humidity_dev: LinuxI2CDevice,
    temp_m: f64,
    temp_c: f64,
    hum_m: f64,
    hum_c: f64,
}

#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    I2CError(LinuxI2CError),
}

pub type SenseHatResult<T> = Result<T, SenseHatError>;

// Registers for the HT221 humidity sensor
// const HTS221_WHO_AM_I: u8 = 0x0f;
const HTS221_AV_CONF: u8 = 0x10;
const HTS221_CTRL1: u8 = 0x20;
// const HTS221_CTRL2: u8 = 0x21;
// const HTS221_CTRL3: u8 = 0x22;
const HTS221_STATUS: u8 = 0x27;
const HTS221_HUMIDITY_OUT_L: u8 = 0x28;
const HTS221_HUMIDITY_OUT_H: u8 = 0x29;
const HTS221_TEMP_OUT_L: u8 = 0x2a;
const HTS221_TEMP_OUT_H: u8 = 0x2b;
const HTS221_H0_H_2: u8 = 0x30;
const HTS221_H1_H_2: u8 = 0x31;
const HTS221_T0_C_8: u8 = 0x32;
const HTS221_T1_C_8: u8 = 0x33;
const HTS221_T1_T0: u8 = 0x35;
const HTS221_H0_T0_OUT: u8 = 0x36;
const HTS221_H1_T0_OUT: u8 = 0x3a;
const HTS221_T0_OUT: u8 = 0x3c;
const HTS221_T1_OUT: u8 = 0x3e;

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

impl SenseHat {
    /// Try and create a new SenseHat object.
    ///
    /// Will open the relevant I2C devices and then attempt to initialise the
    /// chips on the Sense Hat.
    pub fn new() -> SenseHatResult<SenseHat> {
        let mut hat = SenseHat {
            pressure_dev: LinuxI2CDevice::new("/dev/i2c-1", 0x5c)?,
            humidity_dev: LinuxI2CDevice::new("/dev/i2c-1", 0x5f)?,
            temp_m: 0.0,
            temp_c: 0.0,
            hum_m: 0.0,
            hum_c: 0.0,
        };

        hat.init_pressure()?;
        hat.init_humidity()?;

        Ok(hat)
    }

    fn init_pressure(&mut self) -> SenseHatResult<()> {
        // Init sequence from https://github.com/RPi-Distro/RTIMULib
        self.pressure_dev.smbus_write_byte_data(LPS25H_CTRL_REG_1, 0xc4)?;
        self.pressure_dev.smbus_write_byte_data(LPS25H_RES_CONF, 0x05)?;
        self.pressure_dev.smbus_write_byte_data(LPS25H_FIFO_CTRL, 0xc0)?;
        self.pressure_dev.smbus_write_byte_data(LPS25H_CTRL_REG_2, 0x40)?;
        Ok(())
    }

    fn init_humidity(&mut self) -> SenseHatResult<()> {
        // Init
        self.humidity_dev.smbus_write_byte_data(HTS221_CTRL1, 0x87)?;
        self.humidity_dev.smbus_write_byte_data(HTS221_AV_CONF, 0x1b)?;
        // Get cal
        let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_T1_T0)? as u16;
        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_T0_C_8)? as u16;
        let t0_c_8 = ((raw2 & 0x03) << 8) | raw1;
        let t0 = (t0_c_8 as f64) / 8.0;
        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_T1_C_8)? as u16;
        let t1_c_8 = ((raw2 & 0x0C) << 6) | raw1;
        let t1 = (t1_c_8 as f64) / 8.0;

        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_T0_OUT)? as u16;
        let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_T0_OUT + 1)? as u16;
        let t0_out = ((raw2 << 8) | raw1) as f64;

        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_T1_OUT)? as u16;
        let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_T1_OUT + 1)? as u16;
        let t1_out = ((raw2 << 8) | raw1) as f64;

        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_H0_H_2)?;
        let h0 = (raw1 as f64) / 2.0;

        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_H1_H_2)?;
        let h1 = (raw1 as f64) / 2.0;

        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_H0_T0_OUT)? as u16;
        let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_H0_T0_OUT + 1)? as u16;
        let h0_t0_out = ((raw2 << 8) | raw1) as f64;

        let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_H1_T0_OUT)? as u16;
        let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_H1_T0_OUT + 1)? as u16;
        let h1_t0_out = ((raw2 << 8) | raw1) as f64;

        self.temp_m = (t1 - t0) / (t1_out - t0_out);
        self.temp_c = t0 - (self.temp_m * t0_out);
        self.hum_m = (h1 - h0) / (h1_t0_out - h0_t0_out);
        self.hum_c = h0 - (self.hum_m * h0_t0_out);

        Ok(())
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        let status = self.pressure_dev.smbus_read_byte_data(LPS25H_STATUS_REG)?;
        if (status & 1) != 0 {
            let raw1 = self.pressure_dev.smbus_read_byte_data(LPS25H_TEMP_OUT_L)? as u16;
            let raw2 = self.pressure_dev.smbus_read_byte_data(LPS25H_TEMP_OUT_H)? as u16;
            let raw_total = ((raw2 << 8) + raw1) as i16;
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
            let raw1 = self.pressure_dev.smbus_read_byte_data(LPS25H_PRESS_OUT_XL)? as u32;
            let raw2 = self.pressure_dev.smbus_read_byte_data(LPS25H_PRESS_OUT_L)? as u32;
            let raw3 = self.pressure_dev.smbus_read_byte_data(LPS25H_PRESS_OUT_H)? as u32;
            let raw_total: u32 = (raw3 << 16) + (raw2 << 8) + raw1;
            let hectopascals = (raw_total as f64) / 4096.0;
            Ok(Pressure::from_hectopascals(hectopascals))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Temperature reading from the humidity sensor. It's more
    /// accurate than the barometer (+/- 0.5 degrees C), but over a smaller
    /// range.
    pub fn get_temperature_from_humidity(&mut self) -> SenseHatResult<Temperature> {
        let status = self.humidity_dev.smbus_read_byte_data(HTS221_STATUS)?;
        if (status & 1) != 0 {
            let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_TEMP_OUT_L)? as u16;
            let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_TEMP_OUT_H)? as u16;
            let raw_total = ((raw2 << 8) + raw1) as i16;
            let celcius = ((raw_total as f64) * self.temp_m) + self.temp_c;
            Ok(Temperature::from_celsius(celcius))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a RelativeHumidity value in percent between 0 and 100
    pub fn get_humidity(&mut self) -> SenseHatResult<RelativeHumidity> {
        let status = self.humidity_dev.smbus_read_byte_data(HTS221_STATUS)?;
        if (status & 2) != 0 {
            let raw1 = self.humidity_dev.smbus_read_byte_data(HTS221_HUMIDITY_OUT_L)? as u16;
            let raw2 = self.humidity_dev.smbus_read_byte_data(HTS221_HUMIDITY_OUT_H)? as u16;
            let raw_total = ((raw2 << 8) + raw1) as i16;
            let percent = ((raw_total as f64) * self.hum_m) + self.hum_c;
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

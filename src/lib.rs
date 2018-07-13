//! # A driver for the Raspberry Pi Sense HAT
//!
//! The [Sense HAT](https://www.raspberrypi.org/products/sense-hat/) is a
//! sensor board for the Raspberry Pi. It features an LED matrix, a humidity
//! and temperature sensor, a pressure and temperature sensor and a gyroscope.
//!
//! Supported components:
//!
//! * Humidity and Temperature Sensor (an HTS221)
//! * Pressure and Temperature Sensor (a LPS25H)
//! * Gyroscope (an LSM9DS1, requires the RTIMU library)
//!
//! Currently unsupported components:
//!
//! * LED matrix
//! * Joystick

extern crate byteorder;
extern crate i2cdev;
extern crate measurements;

#[cfg(feature = "rtimu")]
extern crate libc;

#[cfg(feature = "led-matrix")]
extern crate sensehat_screen;

mod hts221;
mod lps25h;
mod rh;

pub use measurements::Angle;
pub use measurements::Pressure;
pub use measurements::Temperature;
pub use rh::RelativeHumidity;

use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

#[cfg(feature = "rtimu")]
mod lsm9ds1;

#[cfg(not(feature = "rtimu"))]
mod lsm9ds1_dummy;
#[cfg(not(feature = "rtimu"))]
use lsm9ds1_dummy as lsm9ds1;

/// Represents an orientation from the IMU
#[derive(Debug, Copy, Clone)]
pub struct Orientation {
    pub roll: Angle,
    pub pitch: Angle,
    pub yaw: Angle,
}

/// Represents a 3D vector
#[derive(Debug, Copy, Clone)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A collection of all the data from the IMU
#[derive(Debug, Default)]
struct ImuData {
    timestamp: u64,
    fusion_pose: Option<Orientation>,
    gyro: Option<Vector3D>,
    accel: Option<Vector3D>,
    compass: Option<Vector3D>,
    pressure: Option<f64>,
    temperature: Option<f64>,
    humidity: Option<f64>,
}

/// Represents the Sense HAT itself
pub struct SenseHat<'a> {
    /// LPS25H pressure sensor
    pressure_chip: lps25h::Lps25h<LinuxI2CDevice>,
    /// HTS221 humidity sensor
    humidity_chip: hts221::Hts221<LinuxI2CDevice>,
    /// LSM9DS1 IMU device
    accelerometer_chip: lsm9ds1::Lsm9ds1<'a>,
    /// Cached data
    data: ImuData,
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
    /// chips on the Sense HAT.
    pub fn new() -> SenseHatResult<SenseHat<'a>> {
        Ok(SenseHat {
            humidity_chip: hts221::Hts221::new(LinuxI2CDevice::new("/dev/i2c-1", 0x5f)?)?,
            pressure_chip: lps25h::Lps25h::new(LinuxI2CDevice::new("/dev/i2c-1", 0x5c)?)?,
            accelerometer_chip: lsm9ds1::Lsm9ds1::new()?,
            data: ImuData::default(),
        })
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        let status = self.pressure_chip.status()?;
        if (status & 1) != 0 {
            Ok(Temperature::from_celsius(
                self.pressure_chip.get_temp_celcius()?
            ))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Pressure value from the barometer
    pub fn get_pressure(&mut self) -> SenseHatResult<Pressure> {
        let status = self.pressure_chip.status()?;
        if (status & 2) != 0 {
            Ok(Pressure::from_hectopascals(
                self.pressure_chip.get_pressure_hpa()?
            ))
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

    /// Returns a vector representing the current orientation, using all
    /// three sensors.
    pub fn get_orientation(&mut self) -> SenseHatResult<Orientation> {
        self.accelerometer_chip.set_fusion();
        if self.accelerometer_chip.imu_read() {
            self.data = self.accelerometer_chip.get_imu_data()?;
        }
        match self.data.fusion_pose {
            Some(o) => Ok(o),
            None => Err(SenseHatError::NotReady)
        }
    }

    /// Get the compass heading (ignoring gyro and magnetometer)
    pub fn get_compass(&mut self) -> SenseHatResult<Angle> {
        self.accelerometer_chip.set_compass_only();
        if self.accelerometer_chip.imu_read() {
            // Don't cache this data
            let data = self.accelerometer_chip.get_imu_data()?;
            match data.fusion_pose {
                Some(o) => Ok(o.yaw),
                None => Err(SenseHatError::NotReady)
            }
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a vector representing the current orientation using only
    /// the gyroscope.
    pub fn get_gyro(&mut self) -> SenseHatResult<Orientation> {
        self.accelerometer_chip.set_gyro_only();
        if self.accelerometer_chip.imu_read() {
            let data = self.accelerometer_chip.get_imu_data()?;
            match data.fusion_pose {
                Some(o) => Ok(o),
                None => Err(SenseHatError::NotReady)
            }
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a vector representing the current orientation using only
    /// the accelerometer.
    pub fn get_accel(&mut self) -> SenseHatResult<Orientation> {
        self.accelerometer_chip.set_accel_only();
        if self.accelerometer_chip.imu_read() {
            let data = self.accelerometer_chip.get_imu_data()?;
            match data.fusion_pose {
                Some(o) => Ok(o),
                None => Err(SenseHatError::NotReady)
            }
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a vector representing the current acceleration in Gs.
    pub fn get_accel_raw(&mut self) -> SenseHatResult<Vector3D> {
        self.accelerometer_chip.set_accel_only();
        if self.accelerometer_chip.imu_read() {
            self.data = self.accelerometer_chip.get_imu_data()?;
        }
        match self.data.accel {
            Some(a) => Ok(a),
            None => Err(SenseHatError::NotReady)
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

// End of file

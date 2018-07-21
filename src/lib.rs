//! # Rust support for the Raspberry Pi Sense HAT.
//!
//! The [Raspberry Pi Sense
//! HAT](https://www.raspberrypi.org/products/sense-hat/) is a sensor board
//! for the Raspberry Pi. It features an LED matrix, a humidity and
//! temperature sensor, a pressure and temperature sensor, a joystick and a
//! gyroscope. See <https://www.raspberrypi.org/products/sense-hat/> for
//! details on the Sense HAT.
//!
//! See <https://github.com/RPi-Distro/python-sense-hat> for the official
//! Python driver. This one tries to follow the same API as the Python
//! version.
//!
//! See <https://github.com/thejpster/pi-workshop-rs/> for some workshop
//! materials which use this driver.
//!
//! ## Supported components:
//!
//! * Humidity and Temperature Sensor (an HTS221)
//! * Pressure and Temperature Sensor (a LPS25H)
//! * Gyroscope (an LSM9DS1, requires the RTIMU library)
//! * LED matrix (partial support for scrolling text only)
//!
//! ## Currently unsupported components:
//!
//! * Joystick
//!
//! ## Example use
//!
//! ```
//! use sensehat::{Colour, SenseHat};
//! if let Ok(mut hat) = SenseHat::new() {
//!     println!("{:?}", hat.get_pressure());
//!     hat.text("Hi!", Colour::RED, Colour::WHITE).unwrap();
//! }
//! ```

extern crate byteorder;
extern crate i2cdev;
extern crate measurements;
#[cfg(feature = "led-matrix")]
extern crate tint;

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

#[cfg(feature = "led-matrix")]
use sensehat_screen::color::PixelColor;

/// Represents an orientation from the IMU.
#[derive(Debug, Copy, Clone)]
pub struct Orientation {
    pub roll: Angle,
    pub pitch: Angle,
    pub yaw: Angle,
}

/// Represents a 3D vector.
#[derive(Debug, Copy, Clone)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Represents an RGB colour.
#[cfg(feature = "led-matrix")]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Colour(PixelColor);

/// A collection of all the data from the IMU.
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

/// Represents the Sense HAT itself.
pub struct SenseHat<'a> {
    /// LPS25H pressure sensor.
    pressure_chip: lps25h::Lps25h<LinuxI2CDevice>,
    /// HTS221 humidity sensor.
    humidity_chip: hts221::Hts221<LinuxI2CDevice>,
    /// LSM9DS1 IMU device.
    accelerometer_chip: lsm9ds1::Lsm9ds1<'a>,
    /// Cached accelerometer data.
    data: ImuData,
}

/// Errors that this crate can return.
#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    I2CError(LinuxI2CError),
    LSM9DS1Error(lsm9ds1::Error),
    ScreenError,
    CharacterError(std::string::FromUtf16Error),
}

/// A shortcut for Results that can return `T` or `SenseHatError`.
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
            None => Err(SenseHatError::NotReady),
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
                None => Err(SenseHatError::NotReady),
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
                None => Err(SenseHatError::NotReady),
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
                None => Err(SenseHatError::NotReady),
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
            None => Err(SenseHatError::NotReady),
        }
    }

    /// Displays a scrolling message on the LED matrix. Blocks until the
    /// entire message has scrolled past.
    ///
    /// The `fg` and `bg` values set the foreground and background colours.
    /// You can either specify:
    /// * a constant colour like `Colour::WHITE`,
    /// * a string from the [W3C basic keywords](https://www.w3.org/TR/css-color-3/#html4) like `"white"` or `"purple"`, or
    /// * an RGB 8-bit triple like `(0, 0xFF, 0)`.
    #[cfg(feature = "led-matrix")]
    pub fn text<FG, BG>(&mut self, message: &str, fg: FG, bg: BG) -> SenseHatResult<()>
    where
        FG: Into<Colour>,
        BG: Into<Colour>,
    {
        // Connect to our LED Matrix screen.
        let mut screen =
            sensehat_screen::Screen::open("/dev/fb1").map_err(|_| SenseHatError::ScreenError)?;
        // Get the default `FontCollection`.
        let fonts = sensehat_screen::FontCollection::new();
        // Create a sanitized `FontString`.
        let sanitized = fonts.sanitize_str(message)?;
        // Render the `FontString` as a vector of pixel frames.
        let pixel_frames = sanitized.pixel_frames(fg.into().0, bg.into().0);
        // Create a `Scroll` from the pixel frame vector.
        let scroll = sensehat_screen::Scroll::new(&pixel_frames);
        // Consume the `FrameSequence` returned by the `right_to_left` method.
        scroll.right_to_left().for_each(|frame| {
            screen.write_frame(&frame.frame_line());
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        });
        Ok(())
    }

    /// Clears the LED matrix
    #[cfg(feature = "led-matrix")]
    pub fn clear(&mut self) -> SenseHatResult<()> {
        // Connect to our LED Matrix screen.
        let mut screen =
            sensehat_screen::Screen::open("/dev/fb1").map_err(|_| SenseHatError::ScreenError)?;
        // Send a blank image to clear the screen
        const OFF: [u8; 128] = [0x00; 128];
        screen.write_frame(&sensehat_screen::FrameLine::from_slice(&OFF));
        Ok(())
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

impl From<std::string::FromUtf16Error> for SenseHatError {
    fn from(err: std::string::FromUtf16Error) -> SenseHatError {
        SenseHatError::CharacterError(err)
    }
}

#[cfg(feature = "led-matrix")]
impl<'a> Into<Colour> for &'a str {
    fn into(self) -> Colour {
        let rgb = tint::Color::name(self).unwrap();
        Colour(rgb.to_rgb255().into())
    }
}

#[cfg(feature = "led-matrix")]
impl<'a> Into<Colour> for (u8, u8, u8) {
    fn into(self) -> Colour {
        Colour(self.into())
    }
}

#[cfg(feature = "led-matrix")]
impl Colour {
    pub const WHITE: Colour = Colour(PixelColor::WHITE);
    pub const RED: Colour = Colour(PixelColor::RED);
    pub const GREEN: Colour = Colour(PixelColor::GREEN);
    pub const BLUE: Colour = Colour(PixelColor::BLUE);
    pub const BLACK: Colour = Colour(PixelColor::BLACK);
    pub const YELLOW: Colour = Colour(PixelColor::YELLOW);
    pub const MAGENTA: Colour = Colour(PixelColor::MAGENTA);
    pub const CYAN: Colour = Colour(PixelColor::CYAN);
}

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "led-matrix")]
    #[test]
    fn check_colours_string() {
        let colour_str: Colour = "red".into();
        let colour_const: Colour = Colour::RED;
        assert_eq!(colour_str, colour_const);
    }

    #[cfg(feature = "led-matrix")]
    #[test]
    fn check_colours_tuple() {
        let colour_tuple: Colour = (0xFF, 0, 0).into();
        let colour_const: Colour = Colour::RED;
        assert_eq!(colour_tuple, colour_const);
    }
}

// End of file

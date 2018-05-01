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

extern crate sensehat_screen;

mod rh;
mod hts221;
mod lps25h;

pub use measurements::Temperature;
pub use measurements::Pressure;
pub use measurements::Angle;
pub use rh::RelativeHumidity;

pub use sensehat_screen::{FrameLine, PixelFrame, PixelColor, Rotate, Screen};

use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

#[cfg(feature = "rtimu")]
mod lsm9ds1;

#[cfg(not(feature = "rtimu"))]
mod lsm9ds1_dummy;
#[cfg(not(feature = "rtimu"))]
use lsm9ds1_dummy as lsm9ds1;

pub const LED_HEIGHT: u8 = 8;
pub const LED_WIDTH: u8 = 8;
pub const LED_NUM_PIXELS: usize = LED_HEIGHT as usize * LED_WIDTH as usize;

/// Represents a specific pixel position on the LED
#[derive(Debug, Copy, Clone)]
pub struct PixelPosition {
    x: u8,
    y: u8
}

/// How to rotate the image on the LED display
#[derive(Debug, Copy, Clone)]
pub enum Rotation {
    /// Don't rotate image - top is near GPIO pins
    Normal,
    /// Rotate image 90 degrees clockwise - top is near USB ports
    Clockwise90,
    /// Rotate image 180 degrees clockwise - top is near HDMI port
    Clockwise180,
    /// Rotate image 270 degrees clockwise - top is near micro SD card
    Clockwise270
}

/// Represents an orientation from the IMU
#[derive(Debug, Copy, Clone)]
pub struct Orientation {
    pub roll: Angle,
    pub pitch: Angle,
    pub yaw: Angle,
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
    orientation: Orientation,
    /// LED matrix rotation
    rotation: Rotation,
    /// Current LED contents
    image: Image,
    /// Handle to the framebuffer
    screen: Screen
}

/// Errors that this crate can return
#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    PositionOutOfBounds,
    I2CError(LinuxI2CError),
    LSM9DS1Error(lsm9ds1::Error),
    FramebufferError(sensehat_screen::framebuffer::FramebufferError)
}

/// An image on the LED matrix
#[derive(Copy, Clone, Debug)]
pub struct Image(PixelFrame);

/// A shortcut for Results that can return `T` or `SenseHatError`
pub type SenseHatResult<T> = Result<T, SenseHatError>;

/// Draw mode
#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    /// Write this change to the display now
    OutputNow,
    /// Buffer this change internally
    BufferInternally
}

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
            orientation: Orientation {
                roll: Angle::from_degrees(0.0),
                pitch: Angle::from_degrees(0.0),
                yaw: Angle::from_degrees(0.0),
            },
            rotation: Rotation::Normal,
            image: Image(PixelFrame::BLUE),
            screen: Screen::open("/dev/fb1")?
        })
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        let status = self.pressure_chip.status()?;
        if (status & 1) != 0 {
            Ok(Temperature::from_celsius(self.pressure_chip
                .get_temp_celcius()?))
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a Pressure value from the barometer
    pub fn get_pressure(&mut self) -> SenseHatResult<Pressure> {
        let status = self.pressure_chip.status()?;
        if (status & 2) != 0 {
            Ok(Pressure::from_hectopascals(self.pressure_chip
                .get_pressure_hpa()?))
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
            self.orientation = self.accelerometer_chip.get_imu_data()?;
        }
        Ok(self.orientation)
    }

    /// Get the compass heading (ignoring gyro and magnetometer)
    pub fn get_compass(&mut self) -> SenseHatResult<Angle> {
        self.accelerometer_chip.set_compass_only();
        if self.accelerometer_chip.imu_read() {
            // Don't cache this data
            let orientation = self.accelerometer_chip.get_imu_data()?;
            Ok(orientation.yaw)
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a vector representing the current orientation using only
    /// the gyroscope.
    pub fn get_gyro(&mut self) -> SenseHatResult<Orientation> {
        self.accelerometer_chip.set_gyro_only();
        if self.accelerometer_chip.imu_read() {
            let orientation = self.accelerometer_chip.get_imu_data()?;
            Ok(orientation)
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Returns a vector representing the current orientation using only
    /// the accelerometer.
    pub fn get_accel(&mut self) -> SenseHatResult<Orientation> {
        self.accelerometer_chip.set_accel_only();
        if self.accelerometer_chip.imu_read() {
            let orientation = self.accelerometer_chip.get_imu_data()?;
            Ok(orientation)
        } else {
            Err(SenseHatError::NotReady)
        }
    }

    /// Set the LED matrix rotation
    pub fn set_rotation(&mut self, rotation: Rotation, redraw: DrawMode) {
        self.rotation = rotation;
        self.image.rotate_mut(rotation);
        match redraw {
            DrawMode::OutputNow => self.redraw(),
            _ => {}
        }
    }

    /// Get the current LED matrix rotation
    pub fn get_rotation(&self) -> Rotation {
        self.rotation
    }

    /// Set the whole bufferd image.
    pub fn set_pixels(&mut self, image: Image) -> SenseHatResult<()> {
        self.image = image;
        self.redraw();
        Ok(())
    }

    /// Get the whole buffered image.
    pub fn get_pixels(&self) -> Image {
        return self.image.clone()
    }

    /// Set the colour of a single pixel.
    pub fn set_pixel(&mut self, position: PixelPosition, color: PixelColor, redraw: DrawMode) -> SenseHatResult<()> {
        if position.valid() {
            self.image.0[position.pixel()] = color;
            match redraw {
                DrawMode::OutputNow => self.redraw(),
                _ => {}
            }
            Ok(())
        } else {
            Err(SenseHatError::PositionOutOfBounds)
        }
    }

    /// Get the colour of a single pixel.
    pub fn get_pixel(&mut self, position: PixelPosition) -> SenseHatResult<PixelColor> {
        if position.valid() {
            Ok(self.image.0[position.pixel()])
        } else {
            Err(SenseHatError::PositionOutOfBounds)
        }
    }

    /// Scroll a message across the screen. Blocks until completion.
    pub fn show_message(&mut self, message: &str, speed: f32, text: PixelColor, background: PixelColor) -> SenseHatResult<()> {
        println!("Would should show {:?} at {} seconds/frame in {:?}/{:?}", message, speed, text, background);
        Ok(())
    }

    /// Write a single character to the screen
    pub fn show_letter(&mut self, letter: char, text: PixelColor, background: PixelColor) -> SenseHatResult<()> {
        println!("Would should show {:?} in {:?}/{:?}", letter, text, background);
        Ok(())
    }

    /// Clear the display
    pub fn clear(&mut self, color: PixelColor, redraw: DrawMode) -> SenseHatResult<()> {
        self.image = Image([color; LED_NUM_PIXELS].into());
        match redraw {
            DrawMode::OutputNow => self.redraw(),
            _ => {}
        }
        Ok(())
    }

    pub fn redraw(&mut self) {
        let image = self.image.rotate_copy(self.rotation);
        let frame = image.0.frame_line();
        self.screen.write_frame(&frame);
    }
}

impl PixelPosition {
    fn valid(&self) -> bool {
        (self.x < LED_WIDTH) && (self.y < LED_HEIGHT)
    }

    fn pixel(&self) -> usize {
        usize::from(self.x) + (usize::from(self.y) * usize::from(LED_HEIGHT))
    }
}

impl Image {
    fn rotate_mut(&mut self, rotation: Rotation) {
        match rotation {
            Rotation::Normal => {},
            Rotation::Clockwise90 => {
                self.0.rotate(Rotate::Ccw270);
            }
            Rotation::Clockwise180 => {
                self.0.rotate(Rotate::Ccw180);
            }
            Rotation::Clockwise270 => {
                self.0.rotate(Rotate::Ccw90);
            }
        }
    }

    pub fn rotate_copy(&self, rotation: Rotation) -> Image {
        let mut im = *self;
        im.rotate_mut(rotation);
        im
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

impl From<sensehat_screen::framebuffer::FramebufferError> for SenseHatError {
    fn from(err: sensehat_screen::framebuffer::FramebufferError) -> SenseHatError {
        SenseHatError::FramebufferError(err)
    }
}

// End of file

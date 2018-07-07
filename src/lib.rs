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

mod hts221;
mod lps25h;
mod rh;

pub use measurements::Angle;
pub use measurements::Pressure;
pub use measurements::Temperature;
pub use rh::RelativeHumidity;

pub use sensehat_screen::{FrameLine, PixelColor, Screen};

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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PixelPosition {
    x: u8,
    y: u8,
}

#[derive(Debug, Copy, Clone)]
pub enum Translation {
    Clip,
    Wrap,
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
    Clockwise270,
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
    screen: Screen,
}

/// Errors that this crate can return
#[derive(Debug)]
pub enum SenseHatError {
    NotReady,
    GenericError,
    PositionOutOfBounds,
    I2CError(LinuxI2CError),
    LSM9DS1Error(lsm9ds1::Error),
    FramebufferError(sensehat_screen::FramebufferError),
}

/// An image on the LED matrix
#[derive(Clone)]
pub struct Image([PixelColor; LED_NUM_PIXELS]);

/// A shortcut for Results that can return `T` or `SenseHatError`
pub type SenseHatResult<T> = Result<T, SenseHatError>;

/// Draw mode
#[derive(Debug, Copy, Clone)]
pub enum DrawMode {
    /// Write this change to the display now
    OutputNow,
    /// Buffer this change internally
    BufferInternally,
}

impl<'a> SenseHat<'a> {
    /// Try and create a new SenseHat object.
    ///
    /// Will open the relevant I2C devices and then attempt to initialise the
    /// chips on the Sense HAT.
    pub fn new() -> SenseHatResult<SenseHat<'a>> {
        let blue_pixel = PixelColor::new(0, 0, 255);
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
            image: Image([blue_pixel; LED_NUM_PIXELS]),
            screen: Screen::open("/dev/fb1")?,
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
        return self.image.clone();
    }

    /// Set the colour of a single pixel.
    pub fn set_pixel(
        &mut self,
        position: PixelPosition,
        color: PixelColor,
        redraw: DrawMode,
    ) -> SenseHatResult<()> {
        self.image.0[position.pixel()] = color;
        match redraw {
            DrawMode::OutputNow => self.redraw(),
            _ => {}
        }
        Ok(())
    }

    /// Get the colour of a single pixel.
    pub fn get_pixel(&mut self, position: PixelPosition) -> SenseHatResult<PixelColor> {
        Ok(self.image.0[position.pixel()])
    }

    /// Scroll a message across the screen. Blocks until completion.
    pub fn show_message(
        &mut self,
        message: &str,
        speed: f32,
        text: PixelColor,
        background: PixelColor,
    ) -> SenseHatResult<()> {
        println!(
            "Would should show {:?} at {} seconds/frame in {:?}/{:?}",
            message, speed, text, background
        );
        Ok(())
    }

    /// Write a single character to the screen
    pub fn show_letter(
        &mut self,
        letter: char,
        text: PixelColor,
        background: PixelColor,
    ) -> SenseHatResult<()> {
        println!(
            "Would should show {:?} in {:?}/{:?}",
            letter, text, background
        );
        Ok(())
    }

    /// Clear the display
    pub fn clear(&mut self, color: PixelColor, redraw: DrawMode) -> SenseHatResult<()> {
        for pixel in self.image.0.iter_mut() {
            *pixel = color;
        }
        match redraw {
            DrawMode::OutputNow => self.redraw(),
            _ => {}
        }
        Ok(())
    }

    pub fn redraw(&mut self) {
        let image = self.image.rotate_copy(self.rotation);
        let frame = FrameLine::from_pixels(&image.0);
        self.screen.write_frame(&frame);
    }
}

impl PixelPosition {
    pub fn new(x: u8, y: u8) -> Result<PixelPosition, String> {
        if x >= LED_WIDTH {
            Err(format!(
                "X value {} is larger than maximum {}",
                x, LED_WIDTH
            ))
        } else if y >= LED_HEIGHT {
            Err(format!(
                "Y value {} is larger than maximum {}",
                y, LED_HEIGHT
            ))
        } else {
            Ok(PixelPosition { x, y })
        }
    }

    pub fn up(mut self, distance: u8, mode: Translation) -> Self {
        match mode {
            Translation::Clip => {
                if u16::from(self.y) + u16::from(distance) >= u16::from(LED_HEIGHT) {
                    self.y = LED_HEIGHT - 1;
                } else {
                    self.y += distance;
                }
            }
            Translation::Wrap => {
                self.y = (self.y + distance) % LED_HEIGHT;
            }
        }
        self
    }

    pub fn down(mut self, distance: u8, mode: Translation) -> Self {
        match mode {
            Translation::Clip => {
                if i16::from(self.y) - i16::from(distance) < 0 {
                    self.y = 0;
                } else {
                    self.y -= distance;
                }
            }
            Translation::Wrap => {
                self.y = ((i16::from(self.y) - i16::from(distance)) % i16::from(LED_HEIGHT)) as u8;
            }
        }
        self
    }

    pub fn right(mut self, distance: u8, mode: Translation) -> Self {
        match mode {
            Translation::Clip => {
                if u16::from(self.x) + u16::from(distance) >= u16::from(LED_WIDTH) {
                    self.x = LED_WIDTH - 1;
                } else {
                    self.x += distance;
                }
            }
            Translation::Wrap => {
                self.x = (self.x + distance) % LED_WIDTH;
            }
        }
        self
    }

    pub fn left(mut self, distance: u8, mode: Translation) -> Self {
        match mode {
            Translation::Clip => {
                if i16::from(self.x) - i16::from(distance) < 0 {
                    self.x = 0;
                } else {
                    self.x -= distance;
                }
            }
            Translation::Wrap => {
                self.x = ((i16::from(self.x) - i16::from(distance)) % i16::from(LED_WIDTH)) as u8;
            }
        }
        self
    }

    pub fn pixel(&self) -> usize {
        usize::from(self.x) + (usize::from(self.y) * usize::from(LED_HEIGHT))
    }
}

impl Image {
    fn rotate_mut(&mut self, rotation: Rotation) {
        match rotation {
            Rotation::Normal => {}
            Rotation::Clockwise90 => unimplemented!(),
            Rotation::Clockwise180 => unimplemented!(),
            Rotation::Clockwise270 => unimplemented!(),
        }
    }

    fn rotate_copy(&self, rotation: Rotation) -> Image {
        let mut im = self.clone();
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

impl From<sensehat_screen::FramebufferError> for SenseHatError {
    fn from(err: sensehat_screen::FramebufferError) -> SenseHatError {
        SenseHatError::FramebufferError(err)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn make_bad_position_y() {
        assert!(PixelPosition::new(0, 8).is_err());
    }
    #[test]
    fn make_bad_position_x() {
        assert!(PixelPosition::new(8, 0).is_err());
    }
    #[test]
    fn make_bad_position_xy() {
        assert!(PixelPosition::new(8, 8).is_err());
        assert!(PixelPosition::new(100, 100).is_err());
    }
    #[test]
    fn move_right() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(
            PixelPosition::new(1, 0).unwrap(),
            p0.right(1, Translation::Wrap)
        );
    }
    #[test]
    fn move_up() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(
            PixelPosition::new(0, 1).unwrap(),
            p0.up(1, Translation::Wrap)
        );
    }
    #[test]
    fn move_up_right() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(
            PixelPosition::new(1, 1).unwrap(),
            p0.up(1, Translation::Wrap).right(1, Translation::Wrap)
        );
    }
    #[test]
    fn move_up_wrap() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(p0, p0.up(8, Translation::Wrap));
    }
    #[test]
    fn move_right_wrap() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(p0, p0.right(8, Translation::Wrap));
    }
    #[test]
    fn move_left_wrap() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(p0, p0.left(8, Translation::Wrap));
    }
    #[test]
    fn move_down_wrap() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        assert_eq!(p0, p0.down(8, Translation::Wrap));
    }
    #[test]
    fn move_up_clip() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        let p2 = PixelPosition::new(0, 7).unwrap();
        assert_eq!(p2, p0.up(100, Translation::Clip));
    }
    #[test]
    fn move_right_clip() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        let p3 = PixelPosition::new(7, 0).unwrap();
        assert_eq!(p3, p0.right(100, Translation::Clip));
    }
    #[test]
    fn move_up_right_clip() {
        let p0 = PixelPosition::new(0, 0).unwrap();
        let p1 = PixelPosition::new(7, 7).unwrap();
        assert_eq!(
            p1,
            p0.right(100, Translation::Clip).up(100, Translation::Clip)
        );
    }
}

// End of file

//! * Driver for the LSM9DS1 accelerometer
//! See http://www.st.com/en/mems-and-sensors/lsm9ds1.html
//!
//! Driver follows https://github.com/RPi-Distro/python-sense-hat/blob/master/sense_hat/sense_hat.py
//! in how it manages the settings file.
//!
//! NOTE: It turns out handling IMUs is really complicated and involves a ton of vector maths.
//!
//! It is almost certainly easier if we just wrap RTIMULib. We would use the
//! `gcc` crate to compile a C wrapper of the RTIMULib C++ API. We'd then call
//! that unsafe C wrapper here, ensuring that any memory allocations were
//! undone on drop.

#![allow(dead_code)]
#![allow(unused_variables)]

use i2cdev::core::I2CDevice;
use i2cdev::sensors::{Accelerometer, AccelerometerSample};
use byteorder::{ByteOrder, LittleEndian};
use std::env;
use std::fs;


// @TODO registers here
pub const REG_RES_CONF: u8 = 0x10;


const ACT_THS: u8 = 0x04; // r/w 04 00000100 00000000
const ACT_DUR: u8 = 0x05; // r/w 05 00000101 00000000
const INT_GEN_CFG_XL: u8 = 0x06; // r/w 06 00000110 00000000
const INT_GEN_THS_X_XL: u8 = 0x07; // r/w 07 00000111 00000000
const INT_GEN_THS_Y_XL: u8 = 0x08; // r/w 08 00001000 00000000
const INT_GEN_THS_Z_XL: u8 = 0x09; // r/w 09 00001001 00000000
const INT_GEN_DUR_XL: u8 = 0x0A; // r/w 0A 00001010 00000000
const REFERENCE_G: u8 = 0x0B; // r/w 0B 00001011 00000000
const INT1_CTRL: u8 = 0x0C; // r/w 0C 00001100 00000000
const INT2_CTRL: u8 = 0x0D; // r/w 0D 00001101 00000000
const WHO_AM_I: u8 = 0x0F; // r 0F 00001111 01101000
const CTRL_REG1_G: u8 = 0x10; // r/w 10 00010000 00000000
const CTRL_REG2_G: u8 = 0x11; // r/w 11 00010001 00000000
const CTRL_REG3_G: u8 = 0x12; // r/w 12 00010010 00000000
const ORIENT_CFG_G: u8 = 0x13; // r/w 13 00010011 00000000
const INT_GEN_SRC_G: u8 = 0x14; // r 14 00010100 output
const OUT_TEMP_L: u8 = 0x15; // r 15 00010101 output
const OUT_TEMP_H: u8 = 0x16; // r 16 00010110 output
const STATUS_REG: u8 = 0x17; // r 17 00010111 output
const OUT_X_L_G: u8 = 0x18; // r 18 00011000 output
const OUT_X_H_G: u8 = 0x19; // r 19 00011001 output
const OUT_Y_L_G: u8 = 0x1A; // r 1A 00011010 output
const OUT_Y_H_G: u8 = 0x1B; // r 1B 00011011 output
const OUT_Z_L_G: u8 = 0x1C; // r 1C 00011100 output
const OUT_Z_H_G: u8 = 0x1D; // r 1D 00011101 output
const CTRL_REG4: u8 = 0x1E; // r/w 1E 00011110 00111000
const CTRL_REG5_XL: u8 = 0x1F; // r/w 1F 00011111 00111000
const CTRL_REG6_XL: u8 = 0x20; // r/w 20 00100000 00000000
const CTRL_REG7_XL: u8 = 0x21; // r/w 21 00100001 00000000
const CTRL_REG8: u8 = 0x22; // r/w 22 00100010 00000100
const CTRL_REG9: u8 = 0x23; // r/w 23 00100011 00000000
const CTRL_REG10: u8 = 0x24; // r/w 24 00100100 00000000
const INT_GEN_SRC_XL: u8 = 0x26; // r 26 00100110 output
const STATUS_REG2: u8 = 0x27; // r 27 00100111 output
const OUT_X_L_XL: u8 = 0x28; // r 28 00101000 output
const OUT_X_H_XL: u8 = 0x29; // r 29 00101001 output
const OUT_Y_L_XL: u8 = 0x2A; // r 2A 00101010 output
const OUT_Y_H_XL: u8 = 0x2B; // r 2B 00101011 output
const OUT_Z_L_XL: u8 = 0x2C; // r 2C 00101100 output
const OUT_Z_H_XL: u8 = 0x2D; // r 2D 00101101 output
const FIFO_CTRL: u8 = 0x2E; // r/w 2E 00101110 00000000
const FIFO_SRC: u8 = 0x2F; // r 2F 00101111 output
const INT_GEN_CFG_G: u8 = 0x30; // r/w 30 00110000 00000000
const INT_GEN_THS_XH_G: u8 = 0x31; // r/w 31 00110001 00000000
const INT_GEN_THS_XL_G: u8 = 0x32; // r/w 32 00110010 00000000
const INT_GEN_THS_YH_G: u8 = 0x33; // r/w 33 00110011 00000000
const INT_GEN_THS_YL_G: u8 = 0x34; // r/w 34 00110100 00000000
const INT_GEN_THS_ZH_G: u8 = 0x35; // r/w 35 00110101 00000000
const INT_GEN_THS_ZL_G: u8 = 0x36; // r/w 36 00110110 00000000
const INT_GEN_DUR_G: u8 = 0x37; // r/w 37 00110111 00000000


// These are wrong
const ACCEL_RANGE: f32 = 2.0; // +- 2G (with defaults)
const ACCEL_BITS: u8 = 10; // 10-bit resolution

const SETTINGS_HOME_PATH: &'static str = ".config/sense_hat";

pub enum Error<T: I2CDevice> {
    SettingsFileNotFound,
    NoHomeDir,
    IOError(::std::io::Error),
    I2CError(T::Error),
}

pub struct Lsm9ds1<T: I2CDevice + Sized> {
    i2cdev: T,
    /// Fusion type type -
    ///   0 - Null. Use if only sensor data required without fusion
    ///   1 - Kalman STATE4
    ///   2 - RTQF
    fusion_type: i32,
    /// Gyro sample rate -
    ///   0 = 95Hz
    ///   1 = 190Hz
    ///   2 = 380Hz
    ///   3 = 760Hz
    gyro_sample_rate: i32,
    /// Gyro bandwidth -
    ///   0 - 3 but see the LSM9DS1 manual for details
    gyro_bw: i32,
    /// Gyro high pass filter -
    ///   0 - 9 but see the LSM9DS1 manual for details
    gyro_hpf: i32,
    /// Gyro full scale range -
    ///   0 = 250 degrees per second
    ///   1 = 500 degrees per second
    ///   2 = 2000 degrees per second
    gyro_fsr: i32,
    /// Accel sample rate -
    ///   1 = 14.9Hz
    ///   2 = 59.5Hz
    ///   3 = 119Hz
    ///   4 = 238Hz
    ///   5 = 476Hz
    ///   6 = 952Hz
    accel_sample_rate: i32,
    /// Accel full scale range -
    ///   0 = +/- 2g
    ///   1 = +/- 16g
    ///   2 = +/- 4g
    ///   3 = +/- 8g
    accel_fsr: i32,
    /// Accel low pass filter -
    ///   0 = 408Hz
    ///   1 = 211Hz
    ///   2 = 105Hz
    ///   3 = 50Hz
    accel_lpf: i32,
    /// Compass sample rate -
    ///   0 = 0.625Hz
    ///   1 = 1.25Hz
    ///   2 = 2.5Hz
    ///   3 = 5Hz
    ///   4 = 10Hz
    ///   5 = 20Hz
    ///   6 = 40Hz
    ///   7 = 80Hz
    compass_sample_rate: i32,
    /// Compass full scale range -
    ///   0 = +/- 400 uT
    ///   1 = +/- 800 uT
    ///   2 = +/- 1200 uT
    ///   3 = +/- 1600 uT
    compass_fsr: i32, // the compass full scale range
}

impl<T> Lsm9ds1<T>
    where T: I2CDevice + Sized
{
    /// Create a new pressure sensor handle for the given path/addr.
    /// Init sequence from https://github.com/RPi-Distro/RTIMULib
    pub fn new(config_name: &str, i2cdev: T) -> Result<Lsm9ds1<T>, Error<T>> {

        // i2cdev.smbus_write_byte_data(REG_CTRL_REG_2, 0x40)?;

        let mut chip = Lsm9ds1 {
            i2cdev: i2cdev,
            // These defaults come from RTIMULib.ini in sense-hat.deb v1.2
            fusion_type: 2,
            gyro_sample_rate: 2,
            gyro_bw: 1,
            gyro_hpf: 4,
            gyro_fsr: 1,
            accel_sample_rate: 3,
            accel_fsr: 3,
            accel_lpf: 3,
            compass_sample_rate: 5,
            compass_fsr: 0,
        };

        chip.load_config(config_name)?;

        Ok(chip)
    }

    /// @todo this function doesn't actually load the RTIMU config
    fn load_config(&mut self, config_name: &str) -> Result<(), Error<T>> {
        let ini_file = format!("{}.ini", config_name);
        let home_dir = if let Some(path) = env::home_dir() {
            path
        } else {
            return Err(Error::NoHomeDir);
        };
        if !home_dir.exists() {
            fs::create_dir(&home_dir)?;
        }
        let home_file = home_dir.join(&ini_file);
        let home_exists = home_file.is_file();
        let system_file = ::std::path::Path::new("/etc").join(&ini_file);
        let system_exists = system_file.is_file();

        if system_exists && !home_exists {
            fs::copy(system_file, home_file)?;
        }

        // Load $HOME/<SETTINGS_HOME_PATH>

        // Go through the file line by line
        // Look for <key>=<value>

        Ok(())
    }

    /// Obtain the status bitfield from the chip.
    pub fn status(&mut self) -> Result<u8, Error<T>> {
        // self.i2cdev.smbus_read_byte_data(REG_STATUS_REG)
        Ok(0)
    }
}

impl<T> ::std::convert::From<::std::io::Error> for Error<T>
    where T: I2CDevice
{
    fn from(err: ::std::io::Error) -> Error<T> {
        Error::IOError(err)
    }
}

impl<T> Accelerometer for Lsm9ds1<T>
    where T: I2CDevice + Sized
{
    type Error = T::Error;

    fn accelerometer_sample(&mut self) -> Result<AccelerometerSample, T::Error> {
        // datasheet recommends multi-byte read to avoid reading
        // an inconsistent set of data
        let mut buf: [u8; 6] = [0u8; 6];

        // try!(self.i2cdev.write(&[REGISTER_X0]));
        // try!(self.i2cdev.read(&mut buf));

        let x: i16 = LittleEndian::read_i16(&[buf[0], buf[1]]);
        let y: i16 = LittleEndian::read_i16(&[buf[2], buf[3]]);
        let z: i16 = LittleEndian::read_i16(&[buf[4], buf[5]]);
        Ok(AccelerometerSample {
               x: (x as f32 / 1023.0) * (ACCEL_RANGE * 2.0),
               y: (y as f32 / 1023.0) * (ACCEL_RANGE * 2.0),
               z: (z as f32 / 1023.0) * (ACCEL_RANGE * 2.0),
           })
    }
}

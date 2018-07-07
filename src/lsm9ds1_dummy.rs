//! * Dummy Driver for the LSM9DS1 accelerometer
//!
//! This is just a placeholder so the the docs build without RTIMULib.

use super::Orientation;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Error {
    RTIMULibError,
}

pub struct Lsm9ds1<'a> {
    phantom: PhantomData<&'a u32>,
}

impl<'a> Lsm9ds1<'a> {
    /// Uses the `RTIMULib` library.
    pub fn new() -> Result<Lsm9ds1<'a>, Error> {
        Ok(Lsm9ds1 {
            phantom: PhantomData,
        })
    }

    /// Make the IMU do some work. When this function returns true, the IMU
    /// has data we can fetch with `get_imu_data()`.
    pub fn imu_read(&mut self) -> bool {
        false
    }

    pub fn set_fusion(&mut self) {}

    pub fn set_compass_only(&mut self) {}

    pub fn set_gyro_only(&mut self) {}

    pub fn set_accel_only(&mut self) {}

    pub fn get_imu_data(&mut self) -> Result<Orientation, Error> {
        Err(Error::RTIMULibError)
    }
}

//! * Driver for the LSM9DS1 accelerometer
//! See `http://www.st.com/en/mems-and-sensors/lsm9ds1.html`
//!
//! Driver needs to follow `https://github.com/RPi-Distro/python-sense-hat/blob/master/sense_hat/sense_hat.py`
//! in how it manages the settings file.
//!
//! It turns out handling IMUs is really complicated and involves a ton of
//! vector maths. We just wrap RTIMULib. We use the `gcc` crate to compile
//! a C wrapper of the RTIMULib C++ API. We then call that unsafe C wrapper
//! here, ensuring that any memory allocations were undone on drop.

use super::{Orientation, Angle};
use libc;

enum RTIMULibContext {}

extern "C" {
    fn rtimulib_wrapper_create() -> *mut RTIMULibContext;
    fn rtimulib_wrapper_destroy(p_context: *mut RTIMULibContext);
    fn rtimulib_set_sensors(
        p_context: *mut RTIMULibContext,
        gyro: libc::c_int,
        accel: libc::c_int,
        compass: libc::c_int,
    );
    fn rtimulib_wrapper_imu_read(p_context: *mut RTIMULibContext) -> libc::c_int;
    fn rtimulib_wrapper_get_imu_data(
        p_context: *mut RTIMULibContext,
        orientation: *mut COrientation,
    ) -> libc::c_int;
}

#[repr(C)]
struct COrientation {
    x: libc::c_double,
    y: libc::c_double,
    z: libc::c_double,
}


#[derive(Debug)]
pub enum Error {
    RTIMULibError,
}

pub struct Lsm9ds1<'a> {
    rtimulib_ref: &'a mut RTIMULibContext,
}


impl<'a> Lsm9ds1<'a> {
    /// Uses the RTIMULib library.
    pub fn new() -> Result<Lsm9ds1<'a>, Error> {

        let ctx_ref = unsafe {
            let ctx_p = rtimulib_wrapper_create();
            if ctx_p.is_null() {
                return Err(Error::RTIMULibError);
            }
            &mut *ctx_p
        };

        Ok(Lsm9ds1 { rtimulib_ref: ctx_ref })
    }

    /// Make the IMU do some work. When this function returns true, the IMU
    /// has data we can fetch with `get_imu_data()`.
    pub fn imu_read(&mut self) -> bool {
        let result = unsafe { rtimulib_wrapper_imu_read(self.rtimulib_ref) };
        result != 0
    }

    pub fn set_fusion(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 1, 1, 1);
        }
    }

    pub fn set_compass_only(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 0, 0, 1);
        }
    }

    pub fn set_gyro_only(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 1, 0, 0);
        }
    }

    pub fn set_accel_only(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 0, 1, 0);
        }
    }

    pub fn get_imu_data(&mut self) -> Result<Orientation, Error> {
        let mut temp = COrientation {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let result = unsafe { rtimulib_wrapper_get_imu_data(self.rtimulib_ref, &mut temp) };
        if result != 0 {
            Ok(Orientation {
                roll: Angle::from_radians(temp.x),
                pitch: Angle::from_radians(temp.y),
                yaw: Angle::from_radians(temp.z),
            })
        } else {
            Err(Error::RTIMULibError)
        }
    }
}

impl<'a> Drop for Lsm9ds1<'a> {
    fn drop(&mut self) {
        unsafe { rtimulib_wrapper_destroy(self.rtimulib_ref) }
    }
}

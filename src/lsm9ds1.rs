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


use super::OrientationDegrees;
use libc;

enum RTIMULibContext {}

extern "C" {
    fn rtimulib_wrapper_create() -> *mut RTIMULibContext;
    fn rtimulib_wrapper_destroy(p_context: *mut RTIMULibContext);
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
        return result == 0;
    }

    pub fn get_imu_data(&mut self) -> Result<OrientationDegrees, Error> {
        let mut storage = COrientation {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let result = unsafe { rtimulib_wrapper_get_imu_data(self.rtimulib_ref, &mut storage) };
        if result == 1 {
            Ok(OrientationDegrees {
                roll: radians_to_degrees(storage.x),
                pitch: radians_to_degrees(storage.y),
                yaw: radians_to_degrees(storage.z),
            })
        } else {
            Err(Error::RTIMULibError)
        }
    }
}

fn radians_to_degrees(radians: f64) -> f64 {
    360.0 * (radians / (::std::f64::consts::PI * 2.0))
}

impl<'a> Drop for Lsm9ds1<'a> {
    fn drop(&mut self) {
        unsafe { rtimulib_wrapper_destroy(self.rtimulib_ref) }
    }
}

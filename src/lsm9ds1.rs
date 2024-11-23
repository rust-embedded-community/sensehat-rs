//! * Driver for the LSM9DS1 accelerometer
//! See `http://www.st.com/en/mems-and-sensors/lsm9ds1.html`
//!
//! Driver needs to follow `https://github.com/RPi-Distro/python-sense-hat/blob/master/sense_hat/sense_hat.py`
//! in how it manages the settings file.
//!
//! It turns out handling IMUs is really complicated and involves a ton of
//! vector maths. We just wrap `RTIMULib`. We use the `gcc` crate to compile
//! a C wrapper of the `RTIMULib` C++ API. We then call that unsafe C wrapper
//! here, ensuring that any memory allocations were undone on drop.

use std::fmt::Display;

use super::{Angle, ImuData, Orientation, Vector3D};
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
        orientation: *mut CAllData,
    ) -> libc::c_int;
}

#[repr(C)]
#[derive(Default)]
struct CAllData {
    timestamp: libc::uint64_t,
    fusion_pose_valid: libc::c_int,
    fusion_pose: CVector3D,
    gyro_valid: libc::c_int,
    gyro: CVector3D,
    accel_valid: libc::c_int,
    accel: CVector3D,
    compass_valid: libc::c_int,
    compass: CVector3D,
    pressure_valid: libc::c_int,
    pressure: libc::c_double,
    temperature_valid: libc::c_int,
    temperature: libc::c_double,
    humidity_valid: libc::c_int,
    humidity: libc::c_double,
}

#[repr(C)]
#[derive(Default)]
struct CVector3D {
    x: libc::c_double,
    y: libc::c_double,
    z: libc::c_double,
}

#[derive(Debug)]
pub enum Error {
    RTIMULibError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RTIMULibError => write!(f, "RTIMULib error"),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) struct Lsm9ds1<'a> {
    rtimulib_ref: &'a mut RTIMULibContext,
}

impl<'a> Lsm9ds1<'a> {
    /// Uses the `RTIMULib` library.
    pub(crate) fn new() -> Result<Lsm9ds1<'a>, Error> {
        let ctx_ref = unsafe {
            let ctx_p = rtimulib_wrapper_create();
            if ctx_p.is_null() {
                return Err(Error::RTIMULibError);
            }
            &mut *ctx_p
        };

        Ok(Lsm9ds1 {
            rtimulib_ref: ctx_ref,
        })
    }

    /// Make the IMU do some work. When this function returns true, the IMU
    /// has data we can fetch with `get_imu_data()`.
    pub(crate) fn imu_read(&mut self) -> bool {
        let result = unsafe { rtimulib_wrapper_imu_read(self.rtimulib_ref) };
        result != 0
    }

    pub(crate) fn set_fusion(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 1, 1, 1);
        }
    }

    pub(crate) fn set_compass_only(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 0, 0, 1);
        }
    }

    pub(crate) fn set_gyro_only(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 1, 0, 0);
        }
    }

    pub(crate) fn set_accel_only(&mut self) {
        unsafe {
            rtimulib_set_sensors(self.rtimulib_ref, 0, 1, 0);
        }
    }

    pub(crate) fn get_imu_data(&mut self) -> Result<ImuData, Error> {
        let mut temp = CAllData::default();
        let result = unsafe { rtimulib_wrapper_get_imu_data(self.rtimulib_ref, &mut temp) };
        if result != 0 {
            Ok(ImuData {
                timestamp: temp.timestamp,
                fusion_pose: if temp.fusion_pose_valid != 0 {
                    Some(Orientation {
                        roll: Angle::from_radians(temp.fusion_pose.x),
                        pitch: Angle::from_radians(temp.fusion_pose.y),
                        yaw: Angle::from_radians(temp.fusion_pose.z),
                    })
                } else {
                    None
                },
                gyro: if temp.gyro_valid != 0 {
                    Some(Vector3D {
                        x: temp.gyro.x,
                        y: temp.gyro.y,
                        z: temp.gyro.z,
                    })
                } else {
                    None
                },
                accel: if temp.accel_valid != 0 {
                    Some(Vector3D {
                        x: temp.accel.x,
                        y: temp.accel.y,
                        z: temp.accel.z,
                    })
                } else {
                    None
                },
                compass: if temp.compass_valid != 0 {
                    Some(Vector3D {
                        x: temp.compass.x,
                        y: temp.compass.y,
                        z: temp.compass.z,
                    })
                } else {
                    None
                },
                pressure: if temp.pressure_valid != 0 {
                    Some(temp.pressure)
                } else {
                    None
                },
                temperature: if temp.temperature_valid != 0 {
                    Some(temp.temperature)
                } else {
                    None
                },
                humidity: if temp.humidity_valid != 0 {
                    Some(temp.humidity)
                } else {
                    None
                },
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

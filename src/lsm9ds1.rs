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


enum RTIMULibContext {}

extern "C" {
    fn rtimulib_wrapper_create() -> *mut RTIMULibContext;
    fn rtimulib_wrapper_destroy(p_context: *mut RTIMULibContext);
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
}

impl<'a> Drop for Lsm9ds1<'a> {
    fn drop(&mut self) {
        unsafe { rtimulib_wrapper_destroy(self.rtimulib_ref) }
    }
}

extern crate measurements;

use std::fmt;

pub use measurements::Temperature;

pub struct Pressure {
    value: f64,
}

pub struct RelativeHumidity {
    value: f64,
}

pub struct SenseHat {
    _foo: u32,
}

#[derive(Debug)]
pub enum SenseHatError {
    GenericError,
}

pub type SenseHatResult<T> = Result<T, SenseHatError>;

impl SenseHat {
    /// Try and create a new SenseHat object
    pub fn new() -> SenseHatResult<SenseHat> {
        Ok(SenseHat { _foo: 5 })
    }

    /// Returns a Temperature reading from the humidity sensor. It's more
    /// accurate than the barometer (+/- 0.5 degrees C), but over a smaller
    /// range.
    pub fn get_temperature_from_humidity(&mut self) -> SenseHatResult<Temperature> {
        Ok(Temperature::from_celsius(36.7))
    }

    /// Returns a Temperature reading from the barometer.  It's less accurate
    /// than the barometer (+/- 2 degrees C), but over a wider range.
    pub fn get_temperature_from_pressure(&mut self) -> SenseHatResult<Temperature> {
        Ok(Temperature::from_celsius(36.8))
    }

    /// Returns a Pressure value from the barometer
    pub fn get_pressure(&mut self) -> SenseHatResult<Pressure> {
        return Ok(Pressure::from_hectopascals(1000.0));
    }

    /// Returns a RelativeHumidity value in percent between 0 and 100
    pub fn get_humidity(&mut self) -> SenseHatResult<RelativeHumidity> {
        return Ok(RelativeHumidity::from_percent(50.0));
    }
}

impl Pressure {
    /// hectopascals is the same as a millibar
    pub fn from_hectopascals(hectopascals: f64) -> Pressure {
        Pressure { value: hectopascals }
    }

    pub fn as_hectopascals(&self) -> f64 {
        self.value
    }

    pub fn as_millibars(&self) -> f64 {
        self.value
    }

    pub fn as_bar(&self) -> f64 {
        self.value / 1000.0
    }

    pub fn as_pascals(&self) -> f64 {
        self.value * 100.0
    }

    pub fn as_kilopascals(&self) -> f64 {
        self.value / 10.0
    }

    pub fn as_psi(&self) -> f64 {
        self.as_bar() * 14.5038
    }

    pub fn as_atmospheres(&self) -> f64 {
        self.as_bar() / 1.01325
    }
}

impl RelativeHumidity {
    pub fn from_percent(pc: f64) -> RelativeHumidity {
        RelativeHumidity { value: pc }
    }

    pub fn as_percent(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Pressure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} mbar", self.as_hectopascals())
    }
}

impl fmt::Display for RelativeHumidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%", self.as_percent())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn pressure_test() {
        let p = Pressure::from_hectopascals(1000.0);
        assert_eq!(p.as_bar(), 1.0);
        assert_eq!(p.as_psi(), 14.5038);
    }
}

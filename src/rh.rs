//! # Defines a unit for Relative Humidity (which isn't in the measurements crate)

use std::fmt;

/// Represents a relative humidity reading from the humidity sensor
#[derive(Debug, Copy, Clone)]
pub struct RelativeHumidity {
    value: f64,
}

impl RelativeHumidity {
    pub fn from_percent(pc: f64) -> RelativeHumidity {
        RelativeHumidity { value: pc }
    }

    pub fn as_percent(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for RelativeHumidity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.1}%", self.as_percent())
    }
}

// End of file

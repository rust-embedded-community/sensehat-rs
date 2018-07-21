# sensehat-rs

Rust support for the [Raspberry Pi Sense
HAT](https://www.raspberrypi.org/products/sense-hat/). The Sense HAT is a
sensor board for the Raspberry Pi. It features an LED matrix, a humidity and
temperature sensor, a pressure and temperature sensor, a joystick and a
gyroscope. See <https://www.raspberrypi.org/products/sense-hat/> for details
on the Sense HAT.

See <https://github.com/RPi-Distro/python-sense-hat> for the official Python driver. This one tries to follow the same API as the Python version.

See <https://github.com/thejpster/pi-workshop-rs/> for some workshop materials which use this driver.

## Supported components:

* Humidity and Temperature Sensor (an HTS221)
* Pressure and Temperature Sensor (a LPS25H)
* Gyroscope (an LSM9DS1, requires the RTIMU library)
* LED matrix (partial support for scrolling text only)

## Currently unsupported components:

* Joystick

## Example use

```
use sensehat::{Colour, SenseHat};
if let Ok(mut hat) = SenseHat::new() {
    println!("{:?}", hat.get_pressure());
    hat.text("Hi!", Colour::RED, Colour::WHITE);
}
```

extern crate sensehat;

use sensehat::{Fps, SenseHat};

fn main() {
    let mut sense_hat = SenseHat::new().unwrap();
    let temp = sense_hat.get_temperature_from_humidity().unwrap();
    let msg = format!("It's {}", temp);
    loop {
	    sense_hat.show_message(&msg, Fps(1), "green", "black").unwrap();
    }
}

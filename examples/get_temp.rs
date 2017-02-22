extern crate sensehat;

use sensehat::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().unwrap();
    let temp = sense_hat.get_temperature_from_humidity().unwrap();
    println!("It's {} on the humidity sensor", temp);
    let temp = sense_hat.get_temperature_from_pressure().unwrap();
    println!("It's {} on the pressure sensor", temp);
}

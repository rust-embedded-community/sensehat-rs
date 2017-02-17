extern crate sensehat_rs;

use sensehat_rs::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().unwrap();
    let pressure = sense_hat.get_pressure().unwrap();
    println!("The pressure is {}", pressure);
}

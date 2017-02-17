extern crate sensehat_rs;

use sensehat_rs::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().unwrap();
    let temp = sense_hat.get_temperature_from_humidity().unwrap();
    println!("It's {:.2}\u{00B0} Celcius", temp.as_celsius());
}

extern crate sensehat;

use sensehat::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().unwrap();
    let rh = sense_hat.get_humidity().unwrap();
    println!("It's {} relative humidity", rh);
}

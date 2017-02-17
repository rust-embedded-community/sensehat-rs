extern crate sensehat_rs;

use sensehat_rs::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().unwrap();
    let rh = sense_hat.get_humidity().unwrap();
    println!("It's {} relative humidity", rh);
}

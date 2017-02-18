extern crate sensehat_rs;

use sensehat_rs::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense Hat object");
    let temp = sense_hat.get_temperature_from_humidity().expect("Couldn't get temp");
    println!("It's {:.2}\u{00B0} Celcius on the humidity sensor",
             temp.as_celsius());
    let temp = sense_hat.get_temperature_from_pressure().expect("Couldn't get temp");
    println!("It's {:.2}\u{00B0} Celcius on the pressure sensor",
             temp.as_celsius());
    let rh = sense_hat.get_humidity().expect("Couldn't get rh");
    println!("It's {} relative humidity", rh);
    let pressure = sense_hat.get_pressure().expect("Couldn't get pressure");
    println!("The pressure is {}", pressure);
}

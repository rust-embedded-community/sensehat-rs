extern crate sensehat;

use sensehat::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense Hat object");
    let temp = sense_hat
        .get_temperature_from_humidity()
        .expect("Couldn't get temp");
    println!("It's {} on the humidity sensor", temp);
    let temp = sense_hat
        .get_temperature_from_pressure()
        .expect("Couldn't get temp");
    println!("It's {} on the pressure sensor", temp);
    let rh = sense_hat.get_humidity().expect("Couldn't get rh");
    println!("It's {} relative humidity", rh);
    let pressure = sense_hat.get_pressure().expect("Couldn't get pressure");
    println!("The pressure is {}", pressure);
    loop {
        let orientation = sense_hat.get_orientation_degrees().expect("Couldn't get orientation");
        println!("The orientation is {:?}", orientation);
        ::std::thread::sleep(::std::time::Duration::from_millis(500));
    }
}

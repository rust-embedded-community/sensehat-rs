extern crate sensehat;

use sensehat::SenseHat;

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense Hat object");
    let temp = sense_hat.get_temperature_from_humidity().expect(
        "Couldn't get temp",
    );
    println!("It's {} on the humidity sensor", temp);
    let temp = sense_hat.get_temperature_from_pressure().expect(
        "Couldn't get temp",
    );
    println!("It's {} on the pressure sensor", temp);
    let rh = sense_hat.get_humidity().expect("Couldn't get rh");
    println!("It's {} relative humidity", rh);
    let pressure = sense_hat.get_pressure().expect("Couldn't get pressure");
    println!("The pressure is {}", pressure);
    loop {
        let orientation = sense_hat.get_orientation().expect(
            "Couldn't get orientation",
        );
        println!(
            "Fusion orientation: {}, {}, {}",
            orientation.roll,
            orientation.pitch,
            orientation.yaw
        );
        if let Ok(heading) = sense_hat.get_compass() {
            println!("Compass heading  :  {}", heading);
        }
        if let Ok(orientation) = sense_hat.get_gyro() {
            println!(
                "Gyro orientation :  {}, {}, {}",
                orientation.roll,
                orientation.pitch,
                orientation.yaw
            );
        }
        if let Ok(orientation) = sense_hat.get_accel() {
            println!(
                "Accel orientation:  {}, {}, {}",
                orientation.roll,
                orientation.pitch,
                orientation.yaw
            );
        }
        ::std::thread::sleep(::std::time::Duration::from_millis(250));
    }
}

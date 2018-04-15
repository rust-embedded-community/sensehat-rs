extern crate sensehat;

use sensehat::{SenseHat, FrameLine, PixelColor, Screen};

const DARK: PixelColor = PixelColor::BLACK;
const BLUE: PixelColor = PixelColor::BLUE;
const RED: PixelColor = PixelColor::RED;

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense HAT object");
    let mut screen = Screen::open("/dev/fb1").expect("Couldn't find Sense HAT LED matrix");

    let background_pixels = vec![ 
        BLUE, BLUE, DARK, DARK, DARK, DARK, BLUE, BLUE, //
        BLUE, DARK, BLUE, BLUE, BLUE, BLUE, DARK, BLUE, //
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, //
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, //
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, //
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, //
        BLUE, DARK, BLUE, BLUE, BLUE, BLUE, DARK, BLUE, //
        BLUE, BLUE, DARK, DARK, DARK, DARK, BLUE, BLUE, //
    ];

    let background = FrameLine::from_pixels(&background_pixels);

    let right = {
        let mut pxs = background_pixels.clone();
        pxs[31] = RED;
        pxs[39] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let right_up = {
        let mut pxs = background_pixels.clone();
        pxs[14] = RED;
        pxs[23] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let right_down = {
        let mut pxs = background_pixels.clone();
        pxs[47] = RED;
        pxs[54] = RED;
        FrameLine::from_pixels(&pxs)
    };

    let up = {
        let mut pxs = background_pixels.clone();
        pxs[3] = RED;
        pxs[4] = RED;
        FrameLine::from_pixels(&pxs)
    };

    let up_left = {
        let mut pxs = background_pixels.clone();
        pxs[2] = RED;
        pxs[9] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let up_right = {
        let mut pxs = background_pixels.clone();
        pxs[5] = RED;
        pxs[14] = RED;
        FrameLine::from_pixels(&pxs)
    };

    let left = {
        let mut pxs = background_pixels.clone();
        pxs[24] = RED;
        pxs[32] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let left_up = {
        let mut pxs = background_pixels.clone();
        pxs[9] = RED;
        pxs[16] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let left_down = {
        let mut pxs = background_pixels.clone();
        pxs[40] = RED;
        pxs[49] = RED;
        FrameLine::from_pixels(&pxs)
    };

    let down = {
        let mut pxs = background_pixels.clone();
        pxs[59] = RED;
        pxs[60] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let down_left = {
        let mut pxs = background_pixels.clone();
        pxs[49] = RED;
        pxs[58] = RED;
        FrameLine::from_pixels(&pxs)
    };
    let down_right = {
        let mut pxs = background_pixels.clone();
        pxs[54] = RED;
        pxs[61] = RED;
        FrameLine::from_pixels(&pxs)
    };

    screen.write_frame(&background);

    loop {
        if let Ok(needle) = sense_hat.get_compass() {
            // println!("Compass needle @{}", needle.as_degrees());
            match needle.as_degrees() {
                angle if angle > -15.0 && angle <= 15.0 => {
                    screen.write_frame(&right);
                }
                angle if angle > 15.0 && angle <= 45.0 => {
                    screen.write_frame(&right_up);
                }
                angle if angle > 45.0 && angle <= 75.0 => {
                    screen.write_frame(&up_right);
                }
                angle if angle > 75.0 && angle <= 105.0 => {
                    screen.write_frame(&up);
                }
                angle if angle > 105.0 && angle <= 135.0 => {
                    screen.write_frame(&up_left);
                }
                angle if angle > 135.0 && angle <= 165.0 => {
                    screen.write_frame(&left_up);
                }
                angle
                    if (angle > 165.0 && angle <= 180.0) || (angle < -165.0 && angle >= -180.0) =>
                {
                    screen.write_frame(&left);
                }
                angle if angle < -15.0 && angle >= -45.0 => {
                    screen.write_frame(&right_down);
                }
                angle if angle < -45.0 && angle >= -75.0 => {
                    screen.write_frame(&down_right);
                }
                angle if angle < -75.0 && angle >= -105.0 => {
                    screen.write_frame(&down);
                }
                angle if angle < -105.0 && angle >= -135.0 => {
                    screen.write_frame(&down_left);
                }
                angle if angle < -135.0 && angle >= -165.0 => {
                    screen.write_frame(&left_down);
                }
                _ => screen.write_frame(&background),
            }
        }
    }
}

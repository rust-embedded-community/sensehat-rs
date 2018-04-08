extern crate sensehat;

use sensehat::{SenseHat, FrameLine, PixelColor, Screen};

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense HAT object");
    let mut screen = Screen::open("/dev/fb1").expect("Couldn't find Sense HAT LED matrix");

    let blue_px = PixelColor::BLUE;
    let red_px = PixelColor::RED;
    let black_px = PixelColor::BLACK;

    let background_pixels = vec![ 
        blue_px, blue_px, black_px, black_px, black_px, black_px, blue_px, blue_px,
        blue_px, black_px, blue_px, blue_px, blue_px, blue_px, black_px, blue_px,
        black_px, blue_px, blue_px, blue_px, blue_px, blue_px, blue_px, black_px,
        black_px, blue_px, blue_px, blue_px, blue_px, blue_px, blue_px, black_px,
        black_px, blue_px, blue_px, blue_px, blue_px, blue_px, blue_px, black_px,
        black_px, blue_px, blue_px, blue_px, blue_px, blue_px, blue_px, black_px,
        blue_px, black_px, blue_px, blue_px, blue_px, blue_px, black_px, blue_px,
        blue_px, blue_px, black_px, black_px, black_px, black_px, blue_px, blue_px,
    ];

    let right = {
        let mut pxs = background_pixels.clone();
        pxs[31] = red_px;
        pxs[39] = red_px;
        pxs
    };
    let right_up = {
        let mut pxs = background_pixels.clone();
        pxs[14] = red_px;
        pxs[23] = red_px;
        pxs
    };
    let right_down = {
        let mut pxs = background_pixels.clone();
        pxs[47] = red_px;
        pxs[54] = red_px;
        pxs
    };

    let up = {
        let mut pxs = background_pixels.clone();
        pxs[3] = red_px;
        pxs[4] = red_px;
        pxs
    };

    let up_left = {
        let mut pxs = background_pixels.clone();
        pxs[2] = red_px;
        pxs[9] = red_px;
        pxs
    };
    let up_right = {
        let mut pxs = background_pixels.clone();
        pxs[5] = red_px;
        pxs[14] = red_px;
        pxs
    };

    let left = {
        let mut pxs = background_pixels.clone();
        pxs[24] = red_px;
        pxs[32] = red_px;
        pxs
    };
    let left_up = {
        let mut pxs = background_pixels.clone();
        pxs[9] = red_px;
        pxs[16] = red_px;
        pxs
    };
    let left_down = {
        let mut pxs = background_pixels.clone();
        pxs[40] = red_px;
        pxs[49] = red_px;
        pxs
    };

    let down = {
        let mut pxs = background_pixels.clone();
        pxs[59] = red_px;
        pxs[60] = red_px;
        pxs
    };
    let down_left = {
        let mut pxs = background_pixels.clone();
        pxs[49] = red_px;
        pxs[58] = red_px;
        pxs
    };
    let down_right = {
        let mut pxs = background_pixels.clone();
        pxs[54] = red_px;
        pxs[61] = red_px;
        pxs
    };
    let bg_frame = FrameLine::from_pixels(&background_pixels);
    let left_frame = FrameLine::from_pixels(&left);
    let left_up_frame = FrameLine::from_pixels(&left_up);
    let left_down_frame = FrameLine::from_pixels(&left_down);

    let right_frame = FrameLine::from_pixels(&right);
    let right_up_frame = FrameLine::from_pixels(&right_up);
    let right_down_frame = FrameLine::from_pixels(&right_down);

    let up_frame = FrameLine::from_pixels(&up);
    let up_left_frame = FrameLine::from_pixels(&up_left);
    let up_right_frame = FrameLine::from_pixels(&up_right);

    let down_frame = FrameLine::from_pixels(&down);
    let down_left_frame = FrameLine::from_pixels(&down_left);
    let down_right_frame = FrameLine::from_pixels(&down_right);

    screen.write_frame(&bg_frame);

    loop {
        if let Ok(needle) = sense_hat.get_compass() {
            // println!("Compass needle @{}", needle.as_degrees());
            match needle.as_degrees() {
                angle if angle > -15.0 && angle <= 15.0 => {
                    screen.write_frame(&right_frame);
                }
                angle if angle > 15.0 && angle <= 45.0 => {
                    screen.write_frame(&right_up_frame);
                }
                angle if angle > 45.0 && angle <= 75.0 => {
                    screen.write_frame(&up_right_frame);
                }
                angle if angle > 75.0 && angle <= 105.0 => {
                    screen.write_frame(&up_frame);
                }
                angle if angle > 105.0 && angle <= 135.0 => {
                    screen.write_frame(&up_left_frame);
                }
                angle if angle > 135.0 && angle <= 165.0 => {
                    screen.write_frame(&left_up_frame);
                }
                angle
                    if (angle > 165.0 && angle <= 180.0) || (angle < -165.0 && angle >= -180.0) =>
                {
                    screen.write_frame(&left_frame);
                }
                angle if angle < -15.0 && angle >= -45.0 => {
                    screen.write_frame(&right_down_frame);
                }
                angle if angle < -45.0 && angle >= -75.0 => {
                    screen.write_frame(&down_right_frame);
                }
                angle if angle < -75.0 && angle >= -105.0 => {
                    screen.write_frame(&down_frame);
                }
                angle if angle < -105.0 && angle >= -135.0 => {
                    screen.write_frame(&down_left_frame);
                }
                angle if angle < -135.0 && angle >= -165.0 => {
                    screen.write_frame(&left_down_frame);
                }
                _ => screen.write_frame(&bg_frame),
            }
        }
    }
}

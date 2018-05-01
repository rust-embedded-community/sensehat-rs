extern crate sensehat;
extern crate sensehat_screen;

use sensehat::{SenseHat, FrameLine, PixelColor, Screen};
use sensehat_screen::PixelFrame;

const DARK: PixelColor = PixelColor::BLACK;
const BLUE: PixelColor = PixelColor::BLUE;
const RED: PixelColor = PixelColor::RED;

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense HAT object");
    let mut screen = Screen::open("/dev/fb1").expect("Couldn't find Sense HAT LED matrix");

    let background_pixels = vec![ 
        BLUE, BLUE, DARK, DARK, DARK, DARK, BLUE, BLUE, // 0-7
        BLUE, DARK, BLUE, BLUE, BLUE, BLUE, DARK, BLUE, // 8-15
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 16-23
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 24-31
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 32-39
        DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 40-47
        BLUE, DARK, BLUE, BLUE, BLUE, BLUE, DARK, BLUE, // 48-55
        BLUE, BLUE, DARK, DARK, DARK, DARK, BLUE, BLUE, // 56-63
    ];

    let background = PixelFrame::new(&background_pixels);

    let right = {
        let mut pxs = background_pixels.clone();
        pxs[23] = RED.dim(0.5);
        pxs[47] = RED;
        pxs[31] = RED;
        pxs[39] = RED;.dim(0.5)
        PixelFrame::new(&pxs)
    };
    let right_up = {
        let mut pxs = background_pixels.clone();
        pxs[5] = RED.dim(0.5);
        pxs[14] = RED;
        pxs[23] = RED;
        pxs[31] = RED.dim(0.5);
        PixelFrame::new(&pxs)
    };
    let right_down = {
        let mut pxs = background_pixels.clone();
        pxs[39] = RED.dim(0.5);
        pxs[47] = RED;
        pxs[54] = RED;
        pxs[61] = RED.dim(0.5);
        PixelFrame::new(&pxs)
    };

    let up = right.rotate_left();
    let up_left = right_up.rotate_left();
    let up_right = right_down.rotate_left();

    let left = right.rotate_180();
    let left_up = right_down.rotate_180();
    let left_down = right_up.rotate_180();

    let down = up.rotate_180();
    let down_left = up_right.rotate_180();
    let down_right = up_left.rotate_180();

    screen.write_frame(&background.frame_line());

    loop {
        if let Ok(needle) = sense_hat.get_compass() {
            // println!("Compass needle @{}", needle.as_degrees());
            match needle.as_degrees() {
                angle if angle > -15.0 && angle <= 15.0 => {
                    screen.write_frame(&right.frame_line());
                }
                angle if angle > 15.0 && angle <= 45.0 => {
                    screen.write_frame(&right_up.frame_line());
                }
                angle if angle > 45.0 && angle <= 75.0 => {
                    screen.write_frame(&up_right.frame_line());
                }
                angle if angle > 75.0 && angle <= 105.0 => {
                    screen.write_frame(&up.frame_line());
                }
                angle if angle > 105.0 && angle <= 135.0 => {
                    screen.write_frame(&up_left.frame_line());
                }
                angle if angle > 135.0 && angle <= 165.0 => {
                    screen.write_frame(&left_up.frame_line());
                }
                angle
                    if (angle > 165.0 && angle <= 180.0) || (angle < -165.0 && angle >= -180.0) =>
                {
                    screen.write_frame(&left.frame_line());
                }
                angle if angle < -15.0 && angle >= -45.0 => {
                    screen.write_frame(&right_down.frame_line());
                }
                angle if angle < -45.0 && angle >= -75.0 => {
                    screen.write_frame(&down_right.frame_line());
                }
                angle if angle < -75.0 && angle >= -105.0 => {
                    screen.write_frame(&down.frame_line());
                }
                angle if angle < -105.0 && angle >= -135.0 => {
                    screen.write_frame(&down_left.frame_line());
                }
                angle if angle < -135.0 && angle >= -165.0 => {
                    screen.write_frame(&left_down.frame_line());
                }
                _ => screen.write_frame(&background.frame_line()),
            }
        }
    }
}

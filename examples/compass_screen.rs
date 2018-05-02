extern crate sensehat;
extern crate sensehat_screen;

use sensehat::{Image, PixelColor, Rotation, SenseHat};

const DARK: PixelColor = PixelColor::BLACK;
const BLUE: PixelColor = PixelColor::BLUE;
const RED: PixelColor = PixelColor::RED;

const BACKGROUND: [PixelColor; 64] = [
    BLUE, BLUE, DARK, DARK, DARK, DARK, BLUE, BLUE, // 0-7
    BLUE, DARK, BLUE, BLUE, BLUE, BLUE, DARK, BLUE, // 8-15
    DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 16-23
    DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 24-31
    DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 32-39
    DARK, BLUE, BLUE, BLUE, BLUE, BLUE, BLUE, DARK, // 40-47
    BLUE, DARK, BLUE, BLUE, BLUE, BLUE, DARK, BLUE, // 48-55
    BLUE, BLUE, DARK, DARK, DARK, DARK, BLUE, BLUE, // 56-63
];

fn main() {
    let mut sense_hat = SenseHat::new().expect("Couldn't create Sense HAT object");

    let background: Image = BACKGROUND.into();

    let right: Image = {
        let mut pxs = BACKGROUND;
        pxs[23] = RED.dim(0.5);
        pxs[47] = RED;
        pxs[31] = RED;
        pxs[39] = RED.dim(0.5);
        pxs.into()
    };
    let right_up: Image = {
        let mut pxs = BACKGROUND;
        pxs[5] = RED.dim(0.5);
        pxs[14] = RED;
        pxs[23] = RED;
        pxs[31] = RED.dim(0.5);
        pxs.into()
    };
    let right_down: Image = {
        let mut pxs = BACKGROUND;
        pxs[39] = RED.dim(0.5);
        pxs[47] = RED;
        pxs[54] = RED;
        pxs[61] = RED.dim(0.5);
        pxs.into()
    };

    let up = right.rotate_copy(Rotation::Clockwise270);
    let up_left = right_up.rotate_copy(Rotation::Clockwise270);
    let up_right = right_down.rotate_copy(Rotation::Clockwise270);

    let left = right.rotate_copy(Rotation::Clockwise180);
    let left_up = right_down.rotate_copy(Rotation::Clockwise180);
    let left_down = right_up.rotate_copy(Rotation::Clockwise180);

    let down = up.rotate_copy(Rotation::Clockwise180);
    let down_left = up_right.rotate_copy(Rotation::Clockwise180);
    let down_right = up_left.rotate_copy(Rotation::Clockwise180);

    sense_hat.set_pixels(background).unwrap();

    loop {
        if let Ok(needle) = sense_hat.get_compass() {
            // println!("Compass needle @{}", needle.as_degrees());
            match needle.as_degrees() {
                angle if angle > -15.0 && angle <= 15.0 => {
                    sense_hat.set_pixels(right).unwrap();
                }
                angle if angle > 15.0 && angle <= 45.0 => {
                    sense_hat.set_pixels(right_up).unwrap();
                }
                angle if angle > 45.0 && angle <= 75.0 => {
                    sense_hat.set_pixels(up_right).unwrap();
                }
                angle if angle > 75.0 && angle <= 105.0 => {
                    sense_hat.set_pixels(up).unwrap();
                }
                angle if angle > 105.0 && angle <= 135.0 => {
                    sense_hat.set_pixels(up_left).unwrap();
                }
                angle if angle > 135.0 && angle <= 165.0 => {
                    sense_hat.set_pixels(left_up).unwrap();
                }
                angle
                    if (angle > 165.0 && angle <= 180.0) || (angle < -165.0 && angle >= -180.0) =>
                {
                    sense_hat.set_pixels(left).unwrap();
                }
                angle if angle < -15.0 && angle >= -45.0 => {
                    sense_hat.set_pixels(right_down).unwrap();
                }
                angle if angle < -45.0 && angle >= -75.0 => {
                    sense_hat.set_pixels(down_right).unwrap();
                }
                angle if angle < -75.0 && angle >= -105.0 => {
                    sense_hat.set_pixels(down).unwrap();
                }
                angle if angle < -105.0 && angle >= -135.0 => {
                    sense_hat.set_pixels(down_left).unwrap();
                }
                angle if angle < -135.0 && angle >= -165.0 => {
                    sense_hat.set_pixels(left_down).unwrap();
                }
                _ => sense_hat.set_pixels(background).unwrap(),
            }
        }
    }
}

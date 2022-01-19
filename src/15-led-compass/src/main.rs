#![deny(unsafe_code)]
#![no_main]
#![no_std]

use aux15::lsm303agr::Measurement;
use aux15::switch_hal::OutputSwitch;
use m::Float;
// use aux15::Lsm303agr;
#[allow(unused_imports)]
use aux15::{entry, iprint, iprintln, prelude::*};

#[entry]
fn main() -> ! {
    let (leds, mut lsm303agr, mut delay, mut itm) = aux15::init();
    let mut leds = leds.into_array();

    let mut on = None;
    loop {
        let m @ Measurement { x, y, .. } = lsm303agr.mag_data().unwrap();

        // cos(3PI/8)
        const COS3F8PI: f32 = 0.38268343;

        let [x, y] = [x as f32, y as f32];
        let len = (x * x + y * y).sqrt();
        let [x, y] = [x / len, y / len].map(|v| (v < -COS3F8PI) as u8 + (v < COS3F8PI) as u8);

        let next = match (x, y) {
            // 0 = nord
            (1, 0) => Some(0),
            // 1 = nord-est
            (2, 0) => Some(1),
            // 2 = est
            (2, 1) => Some(2),
            // 3 = sud-est
            (2, 2) => Some(3),
            // 4 = sud
            (1, 2) => Some(4),
            // 5 = sud-ovest
            (0, 2) => Some(5),
            // 6 = ovest
            (0, 1) => Some(6),
            // 7 = nord-ovest
            (0, 0) => Some(7),
            _ => None,
        };
        if on != next {
            if let Some(next) = next {
                leds[next].on().unwrap();
                if let Some(prev) = core::mem::replace(&mut on, Some(next)) {
                    leds[prev].off().unwrap();
                }
            }
        }
        // delay.delay_ms(100u16);
    }
}

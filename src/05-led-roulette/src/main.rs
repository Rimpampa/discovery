#![deny(unsafe_code)]
#![no_main]
#![no_std]

use aux5::{entry, Delay, DelayMs, LedArray, OutputSwitch};
use volatile::Volatile;

#[entry]
fn main() -> ! {
    let (mut delay, mut leds): (Delay, LedArray) = aux5::init();

    let mut step_time = 50_u8;
    let v_step_time = Volatile::new(&mut step_time);

    let mut distance = 1;
    let mut v_distance = Volatile::new(&mut distance);

    v_distance.write(v_distance.read().min(leds.len()));
    for led in &mut leds[..v_distance.read()] {
        led.on().ok();
    }

    let mut nxt_on = v_distance.read();
    let mut nxt_off = 0;

    loop {
        delay.delay_ms(v_step_time.read());
        leds[nxt_on].on().ok();

        delay.delay_ms(v_step_time.read());
        leds[nxt_off].off().ok();

        nxt_on += 1;
        if nxt_on == leds.len() {
            nxt_on = 0;
        }

        nxt_off += 1;
        if nxt_off == leds.len() {
            nxt_off = 0;
        }
    }
}

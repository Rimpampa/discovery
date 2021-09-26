#![no_main]
#![no_std]

use aux9::{entry, switch_hal::OutputSwitch, tim6};

#[inline(never)]
fn delay(tim6: &tim6::RegisterBlock, ms: u16) {
    // TIM6 basic timer register block
    // - ARR auto reload register
    //   0:15 auto reload value, once CNT reaches this
    //        value UIF is set and CNT is resetted
    // - SR status register
    //   - UIF update interrupt flag bit, has to be resetted progammatically
    tim6.arr.write(|w| w.arr().bits(ms));
    tim6.cr1.write(|w| w.cen().set_bit());
    while tim6.sr.read().uif().bit_is_clear() {}
    tim6.sr.write(|w| w.uif().clear_bit());
}

#[entry]
fn main() -> ! {
    let (leds, rcc, tim6) = aux9::init();
    let mut leds = leds.into_array();

    const PSC: u16 = 7999; // 8MHz -> 1KHz

    // RCC reset and clock control register block
    // - APB1 peripheral clock enable register
    //   - TIM6EN timer clock enable bit (1 enabled, 0 disabled)
    rcc.apb1enr.write(|w| w.tim6en().set_bit());

    // TIM6 basic timer register block
    // - CR1 control register 1
    //   - CEN clock enable bit (1 enabled, 0 disabled)
    //   - OPM one pulse mode (1 enabled, 0 disabled)
    // - PSC prescaler register
    //   0:15 prescaler value, clock freq. = timer freq. / (PSC + 1)
    tim6.cr1.write(|w| w.cen().clear_bit().opm().set_bit());
    tim6.psc.write(|w| w.psc().bits(PSC));

    let ms = 1000;
    let mut i = 0;
    loop {
        for led in leds.iter_mut().skip(i) {
            led.on().ok();
            delay(tim6, ms);
        }
        for led in leds.iter_mut().take(i) {
            led.on().ok();
            delay(tim6, ms);
        }
        i = match i {
            7 => 0,
            _ => i + 1,
        };
        for led in leds.iter_mut().skip(i + 1) {
            led.off().ok();
            delay(tim6, ms);
        }
        for led in leds.iter_mut().take(i) {
            led.off().ok();
            delay(tim6, ms);
        }
    }
}

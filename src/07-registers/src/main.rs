#![no_main]
#![no_std]

use core::ptr;

#[allow(unused_imports)]
use aux7::{entry, iprint, iprintln, RegisterBlock, ITM};

// Print the current contents of odr
// the output changes even if im not thouching the ODR address
fn iprint_odr(itm: &mut ITM, gpioe: &'static RegisterBlock) {
    iprintln!(&mut itm.stim[0], "ODR = 0x{:032b}", gpioe.odr.read().bits());
}

#[entry]
fn main() -> ! {
    let (mut itm, gpioe) = aux7::init();

    // A magic address!
    // unsafe { const GPIOE_BSRR: u32 = 0x48001018; }
    // 0x48001018 points to a register
    // register are peripherals
    // this register controls GPIO (general purpose input output) pins
    // pins can be set to high (3V) or low (0V)

    // 0..16 set bits
    // 16..32 reset bits

    // Print the initial contents of ODR
    iprint_odr(&mut itm, gpioe);

    // Turn on the "North" LED (red)
    // unsafe { ptr::write_volatile(GPIOE_BSRR as *mut u32, 1 << 9); }
    gpioe.bsrr.write(|w| w.bs9().set_bit());
    iprint_odr(&mut itm, gpioe);

    // Turn on the "East" LED (green)
    // unsafe { ptr::write_volatile(GPIOE_BSRR as *mut u32, 1 << 11); }
    gpioe.bsrr.write(|w| w.bs11().set_bit());
    iprint_odr(&mut itm, gpioe);

    // Turn off the "North" LED
    // unsafe { ptr::write_volatile(GPIOE_BSRR as *mut u32, 1 << (9 + 16)); }
    gpioe.bsrr.write(|w| w.br9().set_bit());
    iprint_odr(&mut itm, gpioe);

    // Turn off the "East" LED
    // unsafe { ptr::write_volatile(GPIOE_BSRR as *mut u32, 1 << (11 + 16)); }
    gpioe.bsrr.write(|w| w.br11().set_bit());
    iprint_odr(&mut itm, gpioe);

    // produces HW error
    // unsafe { ptr::read_volatile(0x4800_1800 as *const u32); }

    #[allow(clippy::empty_loop)]
    loop {}
}

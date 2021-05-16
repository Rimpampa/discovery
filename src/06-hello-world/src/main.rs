#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_imports)]
use aux6::{entry, iprint, iprintln};

#[entry]
fn main() -> ! {
    // instruction trace macrocell
    // let mut itm = aux6::init();

    // iprintln!(&mut itm.stim[0], "Hello, world!");

    // loop {}
    panic!("Oh shit! Oh no...");
}

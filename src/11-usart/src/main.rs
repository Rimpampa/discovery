#![deny(unsafe_code)]
#![no_main]
#![no_std]
#![allow(unused_macros)]

#[allow(unused_imports)]
use aux11::{entry, iprint, iprintln, usart1};

use core::{fmt::Write, iter::once};

macro_rules! uprint {
    ($serial:expr, $($arg:tt)*) => {
        $serial.write_fmt(format_args!($($arg)*)).ok()
    };
}

macro_rules! uprintln {
    ($serial:expr, $fmt:literal) => {
        uprint!($serial, concat!($fmt, "\n"))
    };

    ($serial:expr, $fmt:literal, $($arg:tt)*) => {
        uprint!($serial, concat!($fmt, "\n"), $($arg)*)
    };
}

struct SerialPort {
    usart1: &'static mut usart1::RegisterBlock,
}

impl SerialPort {
    pub fn is_ready(&self) -> bool {
        !self.usart1.isr.read().rxne().bit_is_clear()
    }

    pub fn wait_ready(&self) {
        while !self.is_ready() {}
    }

    pub fn recv(&self) -> u8 {
        self.wait_ready();
        self.usart1.rdr.read().rdr().bits() as u8
    }

    pub fn send(&self, v: u8) {
        while self.usart1.isr.read().txe().bit_is_clear() {}
        self.usart1.tdr.write(|w| w.tdr().bits(v as u16));
    }

    pub fn is_ore(&self) -> bool {
        let ore = self.usart1.isr.read().ore().bit_is_set();
        if ore {
            self.usart1.icr.write(|w| w.orecf().set_bit());
        }
        ore
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &c in s.as_bytes() {
            self.send(c);
        }
        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let (usart1, mono_timer, mut itm) = aux11::init();
    let serial = SerialPort { usart1 };
    let itm = &mut itm.stim[0];

    // string reverse

    let mut buf = [0u8; 32];
    let mut i = 0;

    let mut start = mono_timer.now();
    loop {
        serial.wait_ready();
        while serial.is_ready() {
            let v = serial.recv();
            if i < buf.len() {
                buf[i] = v;
                i += 1;
            }
        }
        if serial.is_ore() {
            iprintln!(itm, "warn ORE");
        }

        if let [_] | [b'\r', ..] = buf[i - 1..] {
            let ([b @ .., b'\r'] | b) = &buf[..i];
            for &c in b.iter().rev().chain(once(&b'\n')) {
                serial.send(c);
            }
            i = 0;
            iprintln!(
                itm,
                "Sending took {}s",
                start.elapsed() * mono_timer.frequency().0
            );
            start = mono_timer.now();
        }
    }
}

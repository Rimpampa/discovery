#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_imports)]
use aux14::{entry, iprint, iprintln, prelude::*};

mod i2c {
    use aux14::i2c1::RegisterBlock;

    macro_rules! declare {
        ($type:ident::$name:ident of $slave:ty: $start:ty => $end:ty) => {
            impl $crate::i2c::$type<$slave> for $name {
                type Start = $start;
                type End = $end;
            }
        };

        ($type:ident::$name:ident of $slave:ty: $addr:expr) => {
            impl $crate::i2c::$type<$slave> for $name {
                const ADDRESS: u8 = $addr;
            }
        };
    }

    pub trait Slave<Of> {
        const ADDRESS: u8;
    }

    pub trait Register<Of> {
        const ADDRESS: u8;
    }

    pub trait RegisterSeq<Of> // where [u8; (Self::End::ADDRESS - Self::Start::ADDRESS) as usize]: Sized
    {
        type Start: Register<Of>;
        type End: Register<Of>;
    }

    pub struct Magnetometer;
    declare!(Slave::Magnetometer of Lsm303Agr: 0b00111100); // lsb not part of the address

    pub struct Accelerometer;
    declare!(Slave::Accelerometer of Lsm303Agr: 0b00110010);

    pub struct WhoAmI;
    declare!(Register::WhoAmI of Magnetometer: 0x4f);
    declare!(Register::WhoAmI of Accelerometer: 0x0f);

    pub struct OutXL;
    declare!(Register::OutXL of Magnetometer: 0x68);
    declare!(Register::OutXL of Accelerometer: 0x28);

    pub struct OutXH;
    declare!(Register::OutXH of Magnetometer: 0x69);
    declare!(Register::OutXH of Accelerometer: 0x29);

    pub struct OutYL;
    declare!(Register::OutYL of Magnetometer: 0x6A);
    declare!(Register::OutYL of Accelerometer: 0x2A);

    pub struct OutYH;
    declare!(Register::OutYH of Magnetometer: 0x6B);
    declare!(Register::OutYH of Accelerometer: 0x2B);

    pub struct OutZL;
    declare!(Register::OutZL of Magnetometer: 0x6C);
    declare!(Register::OutZL of Accelerometer: 0x2C);

    pub struct OutZH;
    declare!(Register::OutZH of Magnetometer: 0x6D);
    declare!(Register::OutZH of Accelerometer: 0x2D);

    pub struct OutX;
    declare!(RegisterSeq::OutX of Magnetometer: OutXL => OutXH);
    declare!(RegisterSeq::OutX of Accelerometer: OutXL => OutXH);

    pub struct OutY;
    declare!(RegisterSeq::OutY of Magnetometer: OutYL => OutYH);
    declare!(RegisterSeq::OutY of Accelerometer: OutYL => OutYH);

    pub struct OutZ;
    declare!(RegisterSeq::OutZ of Magnetometer: OutZL => OutZH);
    declare!(RegisterSeq::OutZ of Accelerometer: OutZL => OutZH);

    pub struct OutXYZ;
    declare!(RegisterSeq::OutXYZ of Magnetometer: OutXL => OutZH);
    declare!(RegisterSeq::OutXYZ of Accelerometer: OutXL => OutZH);

    pub struct Ctrl;
    declare!(Register::Ctrl of Accelerometer: 0x20);

    pub fn reg_addr<T, S: Slave<T>, R: Register<S>>() -> u8 {
        R::ADDRESS
    }

    pub fn reg_seq_size<T, S: Slave<T>, R: RegisterSeq<S>>() -> u8 {
        R::End::ADDRESS - R::Start::ADDRESS
    }

    pub struct Lsm303Agr {
        i2c: &'static RegisterBlock,
    }

    impl Lsm303Agr {
        pub fn new(i2c: &'static RegisterBlock) -> Self {
            Self { i2c }
        }

        pub fn read<S: Slave<Self>, R: Register<S>>(&self) -> u8 {
            self.read_single(S::ADDRESS, R::ADDRESS)
        }

        pub fn write<S: Slave<Self>, R: Register<S>>(&self, byte: u8) {
            self.write_single(S::ADDRESS, R::ADDRESS, byte)
        }

        pub fn read_all<S: Slave<Self>, R: RegisterSeq<S>>(&self, data: &mut [u8]) {
            let len = reg_seq_size::<_, _, R>() as usize;
            assert!(len > 0);
            self.read_many(S::ADDRESS, R::Start::ADDRESS, &mut data[..len])
        }

        /**
        ```text
                     0    1     2   3   4   5     6     7    8   9   10
            +------+--+-------+---+---+---+--+--------+---+----+----+--+
            |Master|ST|SAD + W|   |SUB|   |SR|SADD + R|   |    |NMAK|SP|
            |Slave |  |       |SAK|   |SAK|  |        |SAK|DATA|    |  |
            +------+--+-------+---+---+---+--+--------+---+----+----+--+

            ST = start
            SAD = slave address
            x + W = address + write bit (xxxxxxx0)
            x + R = address + read bit (xxxxxxx1)
            SAK = acknowledge
            SUB = register address
            SR = start (repeated)
            NMAK = no master aknowledge
        ```
        */
        fn read_single(&self, slave: u8, register: u8) -> u8 {
            // STEPS 0-2
            // send the register address to the slave
            self.i2c.cr2.write(|w| {
                // sadd = address of the i2c slave
                w.sadd().bits(slave as u16);
                // rd_wrn clear = write
                w.rd_wrn().clear_bit();
                // nbytes = number of bytes to send (register address is 1 byte)
                w.nbytes().bits(1);
                // autoend clear = set isr.tc when transfer is completed (no stop signal)
                w.autoend().clear_bit();
                // start set = send a start signal
                w.start().set_bit();
                w
            });
            // STEPS 2-4
            // isr.txis set = write ready
            while self.i2c.isr.read().txis().bit_is_clear() {}
            // txdr.txdata = write data register,
            self.i2c.txdr.write(|w| w.txdata().bits(register));
            // isr.tc set = trasmission complete
            while self.i2c.isr.read().tc().bit_is_clear() {}

            // STEPS 5-7
            // Receive the contents of the register we asked for
            self.i2c.cr2.write(|w| {
                // sadd = address of the i2c slave
                w.sadd().bits(slave as u16);
                // rd_wrn set = read
                w.rd_wrn().set_bit();
                // nbytes = number of bytes to read (assume that every register has 1byte size)
                w.nbytes().bits(1);
                // autoend set = send stop signal after nbytes trasferred
                w.autoend().set_bit();
                // start set = send an(other) start signal
                w.start().set_bit();
                w
            });
            // STEPS 7-10
            // isr.rxne set = ready read
            while self.i2c.isr.read().rxne().bit_is_clear() {}
            // rxdr.rxdata = read data register
            self.i2c.rxdr.read().rxdata().bits()
        }

        /**
        ```text
                     0    1     2   3   4   5     6  7    8   9   10
            +------+--+-------+---+---+---+----+---+--+
            |Master|ST|SAD + W|   |SUB|   |DATA|   |SP|
            |Slave |  |       |SAK|   |SAK|    |SAK|  |
            +------+--+-------+---+---+---+---+----+--+

            ST = start
            SAD = slave address
            x + W = address + write bit (xxxxxxx0)
            x + R = address + read bit (xxxxxxx1)
            SAK = acknowledge
            SUB = register address
            SR = start (repeated)
        ```
        */
        fn write_single(&self, slave: u8, register: u8, byte: u8) {
            // STEPS 0-2
            // send the register address to the slave
            self.i2c.cr2.write(|w| {
                // sadd = address of the i2c slave
                w.sadd().bits(slave as u16);
                // rd_wrn clear = write
                w.rd_wrn().clear_bit();
                // nbytes = number of bytes to send (register address is 1 byte)
                w.nbytes().bits(2);
                // autoend clear = set isr.tc when transfer is completed (no stop signal)
                w.autoend().clear_bit();
                // start set = send a start signal
                w.start().set_bit();
                w
            });
            // STEPS 2-4
            // isr.txis set = write ready
            while self.i2c.isr.read().txis().bit_is_clear() {}
            // txdr.txdata = write data register,
            self.i2c.txdr.write(|w| w.txdata().bits(register));
            // isr.txis set = write ready
            while self.i2c.isr.read().txis().bit_is_clear() {}
            // txdr.txdata = write data register,
            self.i2c.txdr.write(|w| w.txdata().bits(byte));
            // isr.tc set = trasmission complete
            while self.i2c.isr.read().tc().bit_is_clear() {}
        }

        /**
        ```text
                     0    1     2   3   4   5     6     7    8   9   10  11   12  13  14   15  16
            +------+--+-------+---+---+---+--+--------+---+----+---+---+----+---+---+----+----+--+
            |Master|ST|SAD + W|   |SUB|   |SR|SADD + R|   |    |MAK|   |    |MAK|   |    |NMAK|SP|
            |Slave |  |       |SAK|   |SAK|  |        |SAK|DATA|   |SAK|DATA|   |SAK|DATA|    |  |
            +------+--+-------+---+---+---+--+--------+---+----+---+---+----+---+---+----+----+--+

            ST = start
            SAD = slave address
            x + W = address + write bit (xxxxxxx0)
            x + R = address + read bit (xxxxxxx1)
            SAK = acknowledge
            SUB = register address
            SR = start (repeated)
            MAK = master aknowledge
            NMAK = no master aknowledge
        ```
        */
        fn read_many(&self, slave: u8, start_reg: u8, data: &mut [u8]) {
            // STEPS 0-2
            // send the register address to the slave
            self.i2c.cr2.write(|w| {
                // sadd = address of the i2c slave
                w.sadd().bits(slave as u16);
                // rd_wrn clear = write
                w.rd_wrn().clear_bit();
                // nbytes = number of bytes to send (register address is 1 byte)
                w.nbytes().bits(1);
                // autoend clear = set isr.tc when transfer is completed (no stop signal)
                w.autoend().clear_bit();
                // start set = send a start signal
                w.start().set_bit();
                w
            });
            // STEPS 2-4
            // isr.txis set = write ready
            while self.i2c.isr.read().txis().bit_is_clear() {}
            // txdr.txdata = write data register,
            self.i2c
                .txdr
                .write(|w| w.txdata().bits(start_reg | 0b10000000));
            // isr.tc set = trasmission complete
            while self.i2c.isr.read().tc().bit_is_clear() {}

            // STEPS 5-7
            // Receive the contents of the register we asked for
            self.i2c.cr2.write(|w| {
                // sadd = address of the i2c slave
                w.sadd().bits(slave as u16);
                // rd_wrn set = read
                w.rd_wrn().set_bit();
                // nbytes = number of bytes to read (assume that every register has 1byte size)
                w.nbytes().bits(data.len() as u8);
                // autoend set = send stop signal after nbytes trasferred
                w.autoend().set_bit();
                // start set = send an(other) start signal
                w.start().set_bit();
                w
            });
            // STEPS 8-16
            for byte in data {
                // isr.rxne set = ready read
                while self.i2c.isr.read().rxne().bit_is_clear() {}
                // rxdr.rxdata = read data register
                *byte = self.i2c.rxdr.read().rxdata().bits()
            }
        }
    }
}

#[entry]
fn main() -> ! {
    use i2c::*;

    let (i2c1, mut delay, mut itm) = aux14::init();

    let lsm = Lsm303Agr::new(i2c1);

    // Expected output: 0x0A - 0b01000000
    let byte = lsm.read::<Magnetometer, WhoAmI>();
    let addr = reg_addr::<_, Magnetometer, WhoAmI>();
    iprintln!(&mut itm.stim[0], "0x{:02X} - 0b{:08b}", addr, byte);

    // Expected output: 0x0A - 0b00110011
    let byte = lsm.read::<Accelerometer, WhoAmI>();
    let addr = reg_addr::<_, Accelerometer, WhoAmI>();
    iprintln!(&mut itm.stim[0], "0x{:02X} - 0b{:08b}", addr, byte);

    // 0100 = ODR (output data rate?) 50Hz
    // 0 = low power mode disabled
    // 1 = z axis enabled
    // 1 = y azis enabled
    // 1 = x axis enabled
    lsm.write::<Accelerometer, Ctrl>(0b01000111);

    loop {
        let mut acc = [0u8; 2 * 3];
        lsm.read_all::<Accelerometer, OutXYZ>(&mut acc);
        let acc =
            [0, 1, 2].map(|v| u16::from_le_bytes([acc[v * 2], acc[v * 2 + 1]]) as i16 / (1 << 6));

        let mut mag = [0u8; 2 * 3];
        lsm.read_all::<Magnetometer, OutXYZ>(&mut mag);
        let mag = [0, 1, 2].map(|v| u16::from_le_bytes([mag[v * 2], mag[v * 2 + 1]]) as i16);

        iprintln!(&mut itm.stim[0], "A: {:>8?} - M: {:>8?}", acc, mag);
        delay.delay_ms(10u32);
    }
}

extern crate rppal;

use rppal::gpio::{Gpio, Error};
use rppal::gpio::Level::{High, Low};
use std::{thread, time};

static SEGPAT: [u32; 16] = [
    0xFC, 0x60, 0xDA, 0xF2, //0 1 2 3
    0x66, 0xB6, 0xBE, 0xE0, //4 5 6 7
    0xFE, 0xF6, 0xEE, 0x3E, //8 9 A b
    0x1A, 0x7A, 0x9E, 0x8E  //c d E F
];

fn main() -> Result<(), Error> {
    let gpio = Gpio::new()?;

    let mut digits = [
        gpio.get(25)?.into_output(),
        gpio.get(8)?.into_output(),
        gpio.get(7)?.into_output(),
        gpio.get(16)?.into_output(),
        gpio.get(20)?.into_output(),
        gpio.get(21)?.into_output(),
    ];

    let mut segs = [
        gpio.get(13)?.into_output(),
        gpio.get(26)?.into_output(),
        gpio.get(5)?.into_output(),
        gpio.get(9)?.into_output(),
        gpio.get(10)?.into_output(),
        gpio.get(19)?.into_output(),
        gpio.get(6)?.into_output(),
        gpio.get(11)?.into_output(),
    ];

    let dig_interval = time::Duration::from_millis(1);
    let dsp_interval = time::Duration::from_millis(1);

    let mut dispseg = |d, s| {
        for x in 0..6 {
            let val = if x == d { High } else { Low };
            digits[x as usize].write(val);
        }
        for y in 0..8 {
            let mask = 1 << (7 - y);
            let seg = if (s & mask) == mask { Low } else { High };
            segs[y as usize].write(seg);
        }
        true
    };

    let mut data = [0, 0, 0, 0, 0, 0];

    loop {
        for i in 0..16 {
            for s in 0..6 {
                data[s as usize] = (i + s) % 16;
            }
            let mut otime = time::Instant::now();
            let mut ptime = otime;
            let mut dp = 0;
            loop {
                let time = time::Instant::now();
                if time.duration_since(ptime).as_millis() > 500 {
                    dp = 1 - dp;
                    ptime = time;
                }
                for d in 0..6 {
                    let ddp = if d % 2 == 0 { dp } else { 1 - dp };
                    dispseg(d, SEGPAT[data[d as usize] as usize] | ddp);
                    //println!("d: {}, data: {}", d, data[d as usize]);
                    thread::sleep(dig_interval);
                }
                if time.duration_since(otime).as_millis() > 1000 {
                    otime = time;
                    break;
                }
                thread::sleep(dsp_interval);
            }
        }
    }
}

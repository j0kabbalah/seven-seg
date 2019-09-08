extern crate rppal;
extern crate ctrlc;

use rppal::gpio::{Gpio, Error, OutputPin};
use rppal::gpio::Level::{High, Low};

use std::{thread, time};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const SEGPAT: [u8; 16] = [
    0xFC, 0x60, 0xDA, 0xF2, //0 1 2 3
    0x66, 0xB6, 0xBE, 0xE0, //4 5 6 7
    0xFE, 0xF6, 0xEE, 0x3E, //8 9 A b
    0x1A, 0x7A, 0x9E, 0x8E  //c d E F
];

struct Seg {
    digits: Vec<OutputPin>,
    segs: Vec<OutputPin>
}

impl Seg {
    fn dispseg(&mut self, d: u8, s: u8) {
        for x in 0u8..6 {
            let val = if x == d { High } else { Low };
            self.digits[x as usize].write(val);
        }
        for y in 0u8..8 {
            let mask = 1 << (7 - y);
            let seg = if (s & mask) == mask { Low } else { High };
            self.segs[y as usize].write(seg);
        }
    }
}


fn main() -> Result<(), Error> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("ERROR setteing CTRL+C handler");

    let gpio = Gpio::new()?;

    let mut seg: Seg = Seg {
        digits: vec![
            gpio.get(25)?.into_output(),
            gpio.get(8)?.into_output(),
            gpio.get(7)?.into_output(),
            gpio.get(16)?.into_output(),
            gpio.get(12)?.into_output(),
            gpio.get(21)?.into_output(),
        ],
        segs: vec![
            gpio.get(13)?.into_output(),
            gpio.get(26)?.into_output(),
            gpio.get(5)?.into_output(),
            gpio.get(9)?.into_output(),
            gpio.get(10)?.into_output(),
            gpio.get(19)?.into_output(),
            gpio.get(6)?.into_output(),
            gpio.get(11)?.into_output(),
        ]
    };

    let dig_interval = time::Duration::from_millis(1);
    let dsp_interval = time::Duration::from_millis(1);

    let mut data = [0, 0, 0, 0, 0, 0];
    let mut otime = time::Instant::now();

    while running.load(Ordering::SeqCst) {
        for i in 0u8..16 {
            for s in 0u8..6 {
                data[s as usize] = (i + s) % 16;
            }
            let mut ptime = otime;
            let mut dp: u8 = 0;
            while running.load(Ordering::SeqCst) {
                let time = time::Instant::now();
                if time.duration_since(ptime).as_millis() > 500 {
                    dp = 1 - dp;
                    ptime = time;
                }
                for d in 0u8..6 {
                    let ddp: u8 = if d % 2 == 0 { dp } else { 1 - dp };
                    let s: u8 = SEGPAT[data[d as usize] as usize] | ddp;
                    seg.dispseg(d, s);
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

    for i in 0u8..6 {
        seg.dispseg(i, 0);
    }
    Ok(())
}

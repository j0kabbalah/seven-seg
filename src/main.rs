extern crate wiringpi;

use wiringpi::pin::Value::{High, Low};
use std::{thread, time};

static SEGPAT: [u32; 16] = [
    0xFC, 0x60, 0xDA, 0xF2, //0 1 2 3
    0x66, 0xB6, 0xBE, 0xE0, //4 5 6 7
    0xFE, 0xF6, 0xEE, 0x3E, //8 9 A b
    0x1A, 0x7A, 0x9E, 0x8E  //c d E F
];

fn main() {
    //Setup WiringPi with its own pin numbering order
    let pi = wiringpi::setup();

    let digits = [
        pi.output_pin(6),
        pi.output_pin(10),
        pi.output_pin(11),
        pi.output_pin(27),
        pi.output_pin(28),
        pi.output_pin(29)
    ];

    let segs = [
	pi.output_pin(23),
	pi.output_pin(25),
	pi.output_pin(21),
	pi.output_pin(13),
	pi.output_pin(12),
	pi.output_pin(24),
	pi.output_pin(22),
	pi.output_pin(14)
    ];

    let dig_interval = time::Duration::from_millis(1);
    let dsp_interval = time::Duration::from_millis(1);

    let dispseg = |d, s| {
	for x in 0..6 {
	    let val = if x == d {
		High
	    } else {
     		Low
	    };
	    digits[x as usize].digital_write(val);
        }
        for y in 0..8 {
            let mask = 1 << (7-y);
	    let seg = if (s & mask) == mask {
		Low
	    } else {
		High
	    };
	    segs[y as usize].digital_write(seg);
	}
	true
    };

    let mut data = [0, 0, 0, 0, 0, 0];

    loop {
	for i in 0..16 {
            for s in 0..6 {
		data[s as usize] = (i + s) % 16;
	    }
	    let mut otime = pi.millis();
	    let mut ptime = otime;
	    let mut dp = 0;
	    loop {
		let time = pi.millis();
                if time >= ptime + 500 {
		    dp = 1 - dp;
		    ptime = time;
		}
	        for d in 0..6 {
		    let ddp = if d % 2 == 0 { dp } else { 1 - dp };
	            dispseg(d, SEGPAT[data[d as usize] as usize] | ddp);
                    //println!("d: {}, data: {}", d, data[d as usize]);
                    thread::sleep(dig_interval);
	        }
		if time >= otime + 1000 {
		    otime = time;
		    break;
		}
	        thread::sleep(dsp_interval);
	    }
        }
    };
}

//    weeble-wobble decentralize time stamping method
//    all rights reserved until further notice:
//
//    weeble:wobble decentralize time stamping method
//    all rights reserved until further notice:
//
//    WEEBLE WOBBLE is a timestamping method using bitcoin blockheight, utc
//    time and a modulus function to create a unique, decentralized, yet
//    verifiable two part time stamp. weeble wobble was originally described in a
//    decentrailized version control proposal known as 0x20bf. The "weeble"
//    component of the time stamping method is simply floor(utc/block_height). The
//    wobble part of the time stamp is where weeble:wobble has more
//    interesting functionality. utc modulus block_height (utc % block_height).
//    utc mod block_height measures the time between bitcoin blocks and can be
//    adjusted to a varying granularity depending on specification needs.
//    weeble functions as a network "minute hand" and wobble functions as a
//    network "second hand" (for example) but can be adjusted to milliseconds etc...
//
//    WEEBLE WOBBLE Copyright (c) 2023 Randy McMillan
//
//    Permission is hereby granted, free of charge, to any person obtaining a copy
//    of this software and associated documentation files (the "Software"), to deal
//    in the Software without restriction, including without limitation the rights
//    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//    copies of the Software, and to permit persons to whom the Software is
//    furnished to do so, subject to the following conditions:
//
//    The above copyright notice and this permission notice shall be included in all
//    copies or substantial portions of the Software.
//
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//    SOFTWARE.
//
//    gnostr Copyright (c) 2023 Randy McMillan Gnostr.org
//
//    Permission is hereby granted, free of charge, to any person obtaining a copy
//    of this software and associated documentation files (the "Software"), to deal
//    in the Software without restriction, including without limitation the rights
//    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//    copies of the Software, and to permit persons to whom the Software is
//    furnished to do so, subject to the following conditions:
//
//    The above copyright notice and this permission notice shall be included in all
//    copies or substantial portions of the Software.
//
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//    SOFTWARE.
//
//    Gnostr.org Copyright (c) 2023 Randy McMillan Gnostr.org
//
//    Permission is hereby granted, free of charge, to any person obtaining a copy
//    of this software and associated documentation files (the "Software"), to deal
//    in the Software without restriction, including without limitation the rights
//    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//    copies of the Software, and to permit persons to whom the Software is
//    furnished to do so, subject to the following conditions:
//
//    The above copyright notice and this permission notice shall be included in all
//    copies or substantial portions of the Software.
//
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//    SOFTWARE.

use gnostr_bins::get_blockheight;
use reqwest::Url;
use std::io::Read;

use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let since_the_epoch = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = since_the_epoch.as_secs();
    let subsec_millis = since_the_epoch.subsec_millis() as u64;
    let now_millis = seconds * 1000 + subsec_millis;
    //println!("now millis: {}", seconds * 1000 + subsec_millis);

    let _ = get_blockheight();
    let url = Url::parse("https://mempool.space/api/blocks/tip/height").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut tmp_string = String::new();
    res.read_to_string(&mut tmp_string).unwrap();
    let tmp_u64 = tmp_string.parse::<u64>().unwrap_or(0);

    //TODO:impl gnostr-weeble_millis
    //let weeble = now_millis as f64 / tmp_u64 as f64;
    let weeble = seconds as f64 / tmp_u64 as f64;
    println!("{}", format!("{}", weeble.floor()));
}

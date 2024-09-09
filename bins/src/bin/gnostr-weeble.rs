//    WEEBLE WOBBLE is a timestamping method using bitcoin blockheight, utc
//    time and a modulus function to create a unique, decentralized, yet
//    verifiable multi part time stamp. weeble wobble was originally described
//    in a decentrailized version control proposal known as 0x20bf.
//
//    weeble=floor(utc_secs/blockheight) (integer)
//
//    Combined with the most current bitcoin blockheight the "weeble" component
//    of the timestamp inherits the unimpeachable and irreversibility of
//    Bitcoin's proof of work and difficulty characteristics.
//
//    blockheight=blocks_tip_height (integer)
//
//    The wobble part of the time stamp is where weeble/blockheight/wobble has
//    more interesting functionality.
//
//    wobble=(utc_secs % block_height) (integer)
//
//    wobble measures the time between bitcoin blocks and can be adjusted
//    to a varying granularity depending on specification needs.
//
//    Conceptually:
//
//    weeble functions as a network "hour hand".
//    block_height functions as a network "minute hand".
//    wobble functions as a network "second hand".
//    wobble_millis for milliseconds etc...
//
//    The WEEBLE WOBBLE timestamping method may be used in many ways.
//    Used with hashing functions may be particularly useful*.
//
//    H(weeble)
//    H(weeble + blockheight)
//    H(weeble + blockheight + wobble)
//
//    H(private_key + H(weeble))
//    H(private_key + H(weeble + blockheight))
//    H(private_key + H(weeble + blockheight + wobble))
//
//    H(private_key + H(weeble))
//    H(blockheight + H(private_key + weeble))
//    H(wobble + H(blockheight + H(private_key + weeble)))
//                      *permutations may fit into broader cryptographic methods
//
//
//    WEEBLE WOBBLE Copyright (c) 2023 Randy McMillan
//
//
//    Permission is hereby granted, free of charge, to any person obtaining a
//    copy of this software and associated documentation files (the "Software"),
//    to deal in the Software without restriction, including without limitation
//    the rights to use, copy, modify, merge, publish, distribute, sublicense,
//    and/or sell copies of the Software, and to permit persons to whom the
//    Software is furnished to do so, subject to the following conditions:
//
//    The above copyright notice and this permission notice shall be included in
//    all copies or substantial portions of the Software.
//
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//    THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//    DEALINGS IN THE SOFTWARE.
//
//    gnostr Copyright (c) 2023 Randy McMillan Gnostr.org
//
//
//    Permission is hereby granted, free of charge, to any person obtaining a
//    copy of this software and associated documentation files (the "Software"),
//    to deal in the Software without restriction, including without limitation
//    the rights to use, copy, modify, merge, publish, distribute, sublicense,
//    and/or sell copies of the Software, and to permit persons to whom the
//    Software is furnished to do so, subject to the following conditions:
//
//    The above copyright notice and this permission notice shall be included in
//    all copies or substantial portions of the Software.
//
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//    THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//    DEALINGS IN THE SOFTWARE.
//
//    Gnostr.org Copyright (c) 2023 Randy McMillan Gnostr.org
//
//
//    Permission is hereby granted, free of charge, to any person obtaining a
//    copy of this software and associated documentation files (the "Software"),
//    to deal in the Software without restriction, including without limitation
//    the rights to use, copy, modify, merge, publish, distribute, sublicense,
//    and/or sell copies of the Software, and to permit persons to whom the
//    Software is furnished to do so, subject to the following conditions:
//
//    The above copyright notice and this permission notice shall be included in
//    all copies or substantial portions of the Software.
//
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
//    THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//    DEALINGS IN THE SOFTWARE.
//
//    weeble-wobble decentralize time stamping method
//
//    all rights reserved until further notice:
//
//    weeble:wobble decentralize time stamping method
//
//    all rights reserved until further notice:
//    weeble/blockheight/wobble decentralize time stamping method
//
//    all rights reserved until further notice:
//
//    WEEBLE WOBBLE is a timestamping method using bitcoin blockheight, utc
//    time and a modulus function to create a unique, decentralized, yet
//    verifiable multi part time stamp. weeble wobble was originally described
//    in a decentrailized version control proposal known as 0x20bf.

//! gnostr-weeble
//!
//! async reqwest to <https://mempool.space/api/blocks/tip/height>
use futures::executor::block_on;
///
/// weeble = (std::time::SystemTime::UNIX_EPOCH (seconds) / bitcoin-blockheight)
///
/// Weebles wobble, but they don't fall down
/// <https://en.wikipedia.org/wiki/Weeble>
///
/// async fn print_weeble()
///
/// let weeble = gnostr_bins::get_weeble();
///
/// print!("{}",weeble.unwrap());
async fn print_weeble() {
    let start = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = start.as_secs();
    let start_subsec_millis = start.subsec_millis() as u64;
    let start_millis = seconds * 1000 + start_subsec_millis;
    #[cfg(debug_assertions)]
    println!("start_millis: {}", start_millis);

    let weeble = gnostr_bins::get_weeble();
    print!("{}", weeble.unwrap());

    let stop = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("get millis error");
    let seconds = stop.as_secs();
    let stop_subsec_millis = stop.subsec_millis() as u64;
    let stop_millis = seconds * 1000 + stop_subsec_millis;
    #[cfg(debug_assertions)]
    println!("\nstop_millis: {}", stop_millis);
    #[cfg(debug_assertions)]
    println!("\ndelta_millis: {}", stop_millis - start_millis);
}

/// fn main()
///
///let future = print_weeble();
///
/// futures::executor::block_on(future);
fn main() {
    let future = print_weeble();
    block_on(future);
}
/// cargo test --bin gnostr-weeble -- --nocapture
#[test]
fn gnostr_weeble() {
    let future = print_weeble(); // Nothing is printed
    block_on(future);
}

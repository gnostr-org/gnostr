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
use gnostr::weeble::{/*weeble, */ weeble_millis_sync, weeble_sync};
use std::env;
///
/// weeble = (std::time::SystemTime::UNIX_EPOCH (seconds) / bitcoin-blockheight)
///
/// Weebles wobble, but they don't fall down
/// <https://en.wikipedia.org/wiki/Weeble>
///

/// fn main()
fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let millis = match args.next() {
        Some(s) => s,
        None => "false".to_string(), // Default value if no argument is provided
    };
    if millis.eq_ignore_ascii_case("true") || millis.eq_ignore_ascii_case("millis") {
        print!("{}", weeble_millis_sync().unwrap().to_string());
    } else {
        print!("{}", weeble_sync().unwrap().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gnostr::get_weeble_async;
    use gnostr::get_weeble_sync;
    use gnostr::global_rt::global_rt;
    use gnostr::weeble::{weeble, weeble_sync};
    /// cargo test --bin gnostr-weeble -- --nocapture
    #[test]
    fn gnostr_weeble() {
        print!("\nweeble:{}\n", weeble().unwrap().to_string());
        print!("\nweeble_sync:{}\n", weeble_sync().unwrap().to_string());
        print!(
            "\nweeble_millis_sync:{}\n",
            weeble_millis_sync().unwrap().to_string()
        );
    }

    #[test]
    fn test_weeble_global_rt() {
        let rt1 = global_rt();
        let rt2 = global_rt();

        // Ensure that the same runtime is returned each time.
        assert!(std::ptr::eq(rt1, rt2));

        // Ensure the runtime is functional by spawning a simple task.
        rt1.block_on(async {
            let _ = tokio::spawn(async {
                println!("gnostr-weeble:main test begin...");
                main();
                println!("\ngnostr-weeble:main test end...");
            })
            .await
            .unwrap();
        });
        // Ensure the runtime is functional by spawning a simple task.
        rt2.block_on(async {
            let _ = tokio::spawn(async {
                println!("gnostr-weeble:main test begin...");
                main();
                println!("\ngnostr-weeble:main test end...");
            })
            .await
            .unwrap();
        });
        rt1.block_on(async {
            let _ = tokio::spawn(async {
                println!("gnostr-weeble:main test begin...");
                let _ = get_weeble_async().await;
                println!("\ngnostr-weeble:main test end...");
            })
            .await
            .unwrap();
        });
        rt1.block_on(async {
            let _ = tokio::spawn(async {
                println!("gnostr-weeble:main test begin...");
                let _ = get_weeble_sync();
                println!("\ngnostr-weeble:main test end...");
            })
            .await
            .unwrap();
        });
    }
}

/// https://docs.rs/inline-c/latest/inline_c/#
///
/// Hello, Gnostr!
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// #
/// # fn main() {
/// #     (assert_c! {
/// #include <stdio.h>
///
/// int main() {
///     printf("Hello, Gnostr!");
///
///     return 0;
/// }
/// #    })
/// #    .success()
/// #    .stdout("Hello, Gnostr!");
/// # }
/// ```

extern crate gnostr_legit;
//extern crate gnostr_bits;
//#[allow(unused_imports)]
//use gnostr_legit::gitminer::Gitminer;
//#[allow(unused_imports)]
//use gnostr_legit::worker::Worker;
//#[allow(unused_imports)]
//use gnostr_legit::repo;

use inline_c::assert_c;

fn main() {
    (assert_c! {
        #include <stdio.h>

        void usage(){ printf("gnostr: usage!!"); };

        int main() {

            int argc = 0;

            if (argc < 2){

                usage();
            
            } else { printf("Hello, Gnostr!"); }

            return 0;
        }
    })
    .success()
    .stdout("Hello, Gnostr!");
    //rust
    println!("Hello, Gnostr!");
}

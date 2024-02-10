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

//extern crate gnostr_bins;
//extern crate gnostr_bits;

use inline_c::assert_c;

fn main() {
    (assert_c! {
        #include <stdio.h>

        int main() {
            printf("Hello, Gnostr!");

            return 0;
        }
    })
    .success()
    .stdout("Hello, Gnostr!");
    //rust
    println!("Hello, Gnostr!");
}

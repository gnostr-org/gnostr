use futures::executor::block_on;
use gnostr_bins::print_watch_list;
fn main() {
    let future = print_watch_list(); // Nothing is printed
    let _ = block_on(future);
}

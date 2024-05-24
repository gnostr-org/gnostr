use futures::executor::block_on;
async fn print_relay_list() {
    print!("{:#}", gnostr_bins::get_relays_public().unwrap());
}
fn main() {
    let future = print_relay_list(); // Nothing is printed
    block_on(future);
}

use gnostr_bins::get_blockheight;

fn main() {
    let bh = get_blockheight();
    println!("{}", bh.unwrap().to_string());
}

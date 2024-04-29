extern crate cc;

fn main() {
    cc::Build::new().file("src/multiply.c").compile("multiply");
    cc::Build::new().file("src/aes.c").compile("aes");
    cc::Build::new().file("src/base64.c").compile("base64");
    cc::Build::new().file("src/sha256.c").compile("sha256");
    cc::Build::new().file("src/nostril.c").compile("nostril");
}

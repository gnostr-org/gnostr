extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/multiply.c")
        .compile("libmultiply.a");
    cc::Build::new().file("src/aes.c").compile("libaes.a");
    cc::Build::new().file("src/base64.c").compile("libbase64.a");
    cc::Build::new().file("src/copyx.c").compile("libcopyx.a");
    cc::Build::new().file("src/sha256.c").compile("libsha256.a");
    cc::Build::new()
        .file("src/nostril.c")
        .compile("libnostril.a");
}

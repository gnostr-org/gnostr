// build.rs

use std::process::Command;

fn main() {
    cc::Build::new()
        .include("../include/cursor.h")
        .include("../include/hex.h")
        .include("../include/base64.h")
        .include("../include/aes.h")
        .include("../include/sha256.h")
        .include("../include/random.h")
        .include("../include/proof.h")
        .include("../include/constants.h")
        .include("../include/struct_key.h")
        .include("../include/struct_args.h")
        .include("../include/struct_nostr_tag.h")
        .include("../include/struct_nostr_event.h")
        .file("src/double.c")
        .file("src/gnostr.c")
        .compile("libgnostr.a");
}


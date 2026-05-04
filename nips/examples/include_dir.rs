use include_dir::{include_dir, Dir};

static PROJECT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR");

static LIB_RS: include_dir::File<'static> = *PROJECT_DIR.get_file("src/lib.rs").unwrap();

fn main(){
let body = LIB_RS.contents_utf8().unwrap();
assert!(body.contains("SOME_INTERESTING_STRING"));
{
    let glob = "**/*.rs";
    for entry in PROJECT_DIR.find(glob).unwrap() {
        println!("Found {}", entry.path().display());
    }
}
}

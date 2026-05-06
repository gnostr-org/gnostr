use std::{process::Command, str};

fn main() {
    let output = Command::new("git")
        .args(["for-each-ref", "--format=%(refname)", "refs/notes"])
        .output()
        .expect("failed to list git notes refs");

    if !output.status.success() {
        eprintln!(
            "{}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        );
        std::process::exit(output.status.code().unwrap_or(1));
    }

    let refs = String::from_utf8_lossy(&output.stdout);
    let mut found = false;

    for ref_name in refs.lines().map(str::trim).filter(|line| !line.is_empty()) {
        found = true;
        println!("== {ref_name} ==");

        let list = Command::new("git")
            .args(["notes", "--ref", ref_name, "list"])
            .output()
            .expect("failed to list git notes");

        if !list.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&list.stderr).trim_end());
            std::process::exit(list.status.code().unwrap_or(1));
        }

        let notes = String::from_utf8_lossy(&list.stdout);
        for line in notes.lines().map(str::trim).filter(|line| !line.is_empty()) {
            let mut fields = line.split_whitespace();
            let _note_oid = fields.next();
            let object_oid = fields.next();

            if let Some(object_oid) = object_oid {
                println!("-- object: {object_oid}");
                let show = Command::new("git")
                    .args(["notes", "--ref", ref_name, "show", object_oid])
                    .output()
                    .expect("failed to show git note");

                if !show.status.success() {
                    eprintln!("{}", String::from_utf8_lossy(&show.stderr).trim_end());
                    std::process::exit(show.status.code().unwrap_or(1));
                }

                print!("{}", String::from_utf8_lossy(&show.stdout));
                println!();
            }
        }
    }

    if !found {
        println!("No git notes refs found.");
    }
}

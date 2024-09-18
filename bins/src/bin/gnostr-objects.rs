use std::ffi::OsString;

use git2::{Oid, Repository};

fn car_cdr(s: &str) -> (&str, &str) {
    for i in 1..5 {
        let r = s.get(0..i);
        match r {
            Some(x) => return (x, &s[i..]),
            None => (),
        }
    }

    (&s[0..0], s)
}

pub fn oid_to_str(oid: &Oid) -> Result<String, &'static str> {
    // Use the format!("{:x}", oid) for full 40-character hex string.
    // For a shorter representation, use oid.short_id() which returns a Result<Buf,
    // Error> and needs further conversion to String.
    Ok(format!("{:#x?}", oid))
    //Ok(format!("{:}", oid))
}

fn main() -> Result<(), git2::Error> {
    // Get path to git repo via command line args or assume current directory
    let repo_root: OsString = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| OsString::from("."));

    // Open git repo
    let repo = Repository::open(&repo_root).expect("Couldn't open repository");

    println!("{:?}", repo.state());

    // Get object database from the repo
    let odb = repo.odb().unwrap();

    // Loop through objects in db
    odb.foreach(|oid| {
        //println!("{}",*oid);
        //println!("{:?}",oid_to_str(oid));
        //format!("{:#x?}", oid)
        //let s:String = "Hello, world!".chars()
        //    .map(|x| match x {
        //        '!' => '?',
        //        'A'..='Z' => 'X',
        //        'a'..='z' => 'x',
        //        _ => x
        //    }).collect();
        //println!("{}", s);// Xxxxx, xxxxx
        let s: String = oid_to_str(oid).expect("REASON");
        let s: String = s
            .chars()
            .map(|x| match x {
                '(' => ' ',
                ')' => ' ',
                'A'..='Z' => ' ',
                //'a'..='z' => 'x',
                _ => x,
            })
            .collect();
        println!("{}", s);
        let first_two_chars: String = s.chars().take(2).collect();
        //let first_two_chars = &first_two_chars[..2];
        //println!("2:First two characters: {}", first_two_chars);
        //println!("{}objects/{}",repo.path().display() ,oid);
        let (_first_char, remainder) = car_cdr(&s);
        let (_second_char, remainder) = car_cdr(&remainder);
        println!(
            "{}objects/{}/{}",
            repo.path().display(),
            first_two_chars,
            remainder
        );
        // Return true because the closure has to return a boolean
        true
    })
    .unwrap();
    Ok(())
}

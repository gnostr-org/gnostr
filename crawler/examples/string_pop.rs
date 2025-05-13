fn main() {
    let mut my_string = String::from("Hello/");
    println!("Original string: '{}'", my_string);

    if my_string.ends_with("/") {
        my_string.pop();
        println!("String after removing '/': '{}'", my_string);
    } else {
        println!("String does not end with '/'.");
    }

    let mut another_string = String::from("World");
    println!("Original string: '{}'", another_string);

    if another_string.ends_with("/") {
        another_string.pop();
        println!("String after removing '/': '{}'", another_string);
    } else {
        println!("String does not end with '/'.");
    }
}

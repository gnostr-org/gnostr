fn main() {
         //f is the scrutinee
    match 'f' {
      //q @ (... OR ...) is a binding pattern
		q @ ('"' | '\'') => {
			println!("{}", q);
		},
		f @ 'f' => {
			println!("{}", f);
		},
		_ => {
			println!("no matches");
		},
    }
}

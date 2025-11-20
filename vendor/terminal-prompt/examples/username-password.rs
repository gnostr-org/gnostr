use terminal_prompt::Terminal;

fn main() -> std::io::Result<()> {
	let mut terminal = Terminal::open()?;
	let username = terminal.prompt("Username: ")?;
	let password = terminal.prompt_sensitive("Password: ")?;
	println!("Username: {username}");
	println!("Password: {password}");
	Ok(())
}

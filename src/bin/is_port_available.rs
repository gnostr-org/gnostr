use std::net::TcpListener;

fn is_port_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => {println!("{}:true", port);true},
        Err(_) => {println!("{}:false",port);false},
    }
}

fn main() {
    let port_to_check = vec![
		8051u16,
		8052u16,
		8053u16,
		8054u16,
		8055u16,
		8056u16,
		8057u16,
		2222u16,
		8080u16,
	];


	for port in port_to_check {


	while !is_port_available(port) {
	if is_port_available(port) {
        println!("Port {} is available.", port);
    } else {
        println!("Port {} is not available.", port);
    }
    }
    }
}

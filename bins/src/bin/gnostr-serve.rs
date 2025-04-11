use std::env;
use std::process::exit;
use std::sync::Arc;

use bytes::Bytes;
use gnostr_bins::router::{Handler, Router};
use gnostr_bins::serve::{AppState, *};
use gnostr_bins::{handler, router};
use hyper::body::to_bytes;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use route_recognizer::Params;
use sysinfo::{get_current_pid, Pid, ProcessExt, System, SystemExt};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() {
    let mut port = 8080; // Default port
    let mut verbose = false;
    #[allow(unused_assignments)]
    let mut port_str: &str = "8080";
    let mut assign_next = false;
    for arg in env::args().skip(1) {
        if assign_next {
            port = arg.parse::<u16>().unwrap();
            break;
        }

        if arg.starts_with("--verbose") || arg.starts_with("-vv") {
            verbose = true;
            //break;
        }
        if arg.starts_with("--port=") || arg.starts_with("-p=") {
            port_str = arg.splitn(2, '=').nth(1).unwrap();
            let parsed_port = port_str.parse::<u16>();
            if let Err(err) = parsed_port {
                eprintln!("Error parsing port: {}", err);
                exit(1);
            }
            port = parsed_port.unwrap();
            break; // Exit after finding the port argument
        }
        if arg.starts_with("--port") || arg.starts_with("-p") {
            //print!("arg={}", arg);
            assign_next = true;
            //print!("assign_next={}", assign_next);
            //break; // Exit after finding the port argument
        }
    }

    //println!("Using port: {}", port);
    let str_port = port.to_string();
    //println!("The string value is: {}", str_port);

    if verbose {
        println!("curl http://localhost:{}/test", &str_port);
        println!("curl http://localhost:{}/params/1234", &str_port);
        println!(
            "curl -X POST http://localhost:{}/send -d '{{\"name\": \"chip\", \"active\": true}}'",
            &str_port
        );
    }
    let some_state = "state".to_string();

    let mut router: Router = Router::new();
    router.get("/test", Box::new(handler::test_handler));
    router.post("/send", Box::new(handler::send_handler));
    router.get("/params/:some_param", Box::new(handler::param_handler));

    let shared_router = Arc::new(router);
    let new_service = make_service_fn(move |_| {
        let app_state = AppState {
            state_thing: some_state.clone(),
        };

        let router_capture = shared_router.clone();
        async {
            Ok::<_, Error>(service_fn(move |req| {
                route(router_capture.clone(), req, app_state.clone())
            }))
        }
    });

    let addr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("address creation works");
    let server = Server::bind(&addr).serve(new_service);

    match get_current_pid() {
        Ok(pid) => {
            let s = System::new_all();
            if let Some(process) = s.process(Pid::from(pid)) {
                println!(
                    "{{\"{}\",\"{}\",\"{}\",\"{}\"}}",
                    process.name(),
                    pid,
                    addr,
                    port
                );
            }
        }
        Err(e) => {
            println!("failed to get current pid: {}", e);
        }
    }
    let _ = server.await;
}

async fn route(
    router: Arc<Router>,
    req: Request<hyper::Body>,
    app_state: AppState,
) -> Result<Response, Error> {
    let found_handler = router.route(req.uri().path(), req.method());
    let resp = found_handler
        .handler
        .invoke(gnostr_bins::serve::Context::new(
            app_state,
            req,
            found_handler.params,
        ))
        .await;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use std::process::{Command, Stdio};

    use sysinfo::{get_current_pid, Pid, Process, System};

    use super::*;

    #[test]
    fn install() {
        //cargo install -q --bin gnostr-serve --path .
        let output = Command::new("cargo")
            .args([
                "install",
                /* "-q", */ "--bin",
                "gnostr-serve",
                "--path",
                ".",
            ])
            .stdout(Stdio::piped())
            //.spawn()
            .output()
            .unwrap();
        //let mut stdout = output.stdout.unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("{:?}", stdout);
    }
    #[test]
    fn curl_test() {
        match get_current_pid() {
            Ok(pid) => {
                println!("current pid: {}", pid);
                let s = System::new_all();
                if let Some(process) = s.process(Pid::from(pid)) {
                    println!("{:?}", process.name());
                }
            }
            Err(e) => {
                println!("failed to get current pid: {}", e);
            }
        }

        //cargo install -q --bin gnostr-serve --path .
        let mut output = Command::new("gnostr-serve")
            .args([""])
            .stdout(Stdio::piped())
            .spawn()
            //.output()
            .unwrap();
        let mut stdout = output.stdout.unwrap();
        //let mut stdout = String::from_utf8(output.stdout).unwrap();
        println!("{:?}", stdout);

        let mut url = "http://localhost:8080/test";
        let output = Command::new("curl")
            .args([url])
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("{}", stdout);

        url = "http://localhost:8080/params/1234";
        let output = Command::new("curl")
            .args([url])
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("{}", stdout);

        url = "http://localhost:8080/send";
        let d = "-d";
        let json = "{\"name\": \"chip\", \"active\": true}";
        let output = Command::new("curl")
            .args([url, d, json])
            .stdout(Stdio::piped())
            .output()
            .unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();
        println!("{}", stdout);

        let system = System::new_all();
        for (_, proc) in system.processes() {
            if proc.name() == "gnostr-serve" {
                proc.kill();
            }
        }
    }
}

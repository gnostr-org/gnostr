//#![deny(warnings)]
use std::sync::Arc;

use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;

use futures_util::{FutureExt, StreamExt};
use warp::Filter;

//dev server
use tiny_http::{Response, Server};

use damus_js::*;

struct WithTemplate<T: Serialize> {
    name: &'static str,
    value: T,
}

fn render<T>(template: WithTemplate<T>, hbs: Arc<Handlebars<'_>>) -> impl warp::Reply
where
    T: Serialize,
{
    let render = hbs
        .render(template.name, &template.value)
        .unwrap_or_else(|err| err.to_string());
    warp::reply::html(render)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    //dev_start();

    let main_js = js::main_js::JSMain::new();
    //println!("{}", main_js.to_string());

    let main_js_bytes: &[u8] = include_bytes!("js/main.js");
    assert_eq!(main_js_bytes, main_js.main_js);

    //println!("First few bytes of main.js: {:?}", &main_js[0..10]);
    if let Ok(main_js_string) = String::from_utf8(main_js_bytes.to_vec()) {
        //println!("main.js content: {}", main_js_string);
    } else {
        //println!("main.js is not valid UTF-8.");
    }

    let template = damus_js::template_html::TemplateHtml::new();
    //"<!DOCTYPE html>
    // <html>
    //   <head>
    //     <title>Warp Handlebars template example</title>
    //   </head>
    //   <body>
    //     <h1>Hello {{user}}!</h1>
    //   </body>
    // </html>";

    let mut hb = Handlebars::new();
    // register the template
    hb.register_template_string("template.html", template.to_string())
        .unwrap();

    // Turn Handlebars instance into a Filter so we can combine it
    // easily with others...
    let hb = Arc::new(hb);

    // Create a reusable closure to render template
    let handlebars = move |with_template| render(with_template, hb.clone());

    //GET /
    let route = warp::get()
        .and(warp::path::end())
        .map(|| WithTemplate {
            name: "template.html",
            value: json!({"user" : "Warp"}),
        })
        .map(handlebars);

    let routes = warp::path("echo")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    // Server 1 (port 3030)
    let server1 = warp::serve(warp::fs::dir(".")).run(([127, 0, 0, 1], 3030));
    println!("server1\n127.0.0.1:3030");
    // Server 2 (port 3031)
    let server2 =
        warp::serve(warp::path::end().map(|| "Hello from server 2!")).run(([127, 0, 0, 1], 3031));
    println!("server1\n127.0.0.1:3031");

    // Server 3 (port 3032)
    let server3 = warp::serve(warp::path("api").map(|| "API response from server 3!"))
        .run(([127, 0, 0, 1], 3032));
    println!("server1\n127.0.0.1:3032");

    let server4 = warp::serve(routes).run(([127, 0, 0, 1], 3033));
    println!("server1\n127.0.0.1:3033");

    let server5 = warp::serve(route).run(([127, 0, 0, 1], 3034));
    println!("server1\n127.0.0.1:3034");

    // Run all servers concurrently
    tokio::select! {
        _ = server1 => {
            println!("Server 1 (3030) exited.");
        }
        _ = server2 => {
            println!("Server 2 (3031) exited.");
        }
        _ = server3 => {
            println!("Server 3 (3032) exited.");
        }
        _ = server4 => {
            println!("Server 3 (3033) exited.");
        }
        _ = server5 => {
            println!("Server 3 (3034) exited.");
        }
    }
    println!("All servers exited.");
}

fn dev_start() {
    let hbs = Arc::new(dev_mode());

    let server = Server::http("127.0.0.1:8080").expect("Failed to start demo server.");
    println!(
        "Edit ./examples/dev_mode/template.hbs\nand request\nhttp://localhost:8080\nto see the change on the fly."
    );

    for req in server.incoming_requests() {
		println!("{:?}", req);
        let result = hbs
            .render("tpl", &json!({"model": "t14s", "brand": "Thinkpad"}))
            .unwrap_or_else(|e| e.to_string());
        req.respond(Response::from_string(result)).unwrap();
    }
}
fn dev_mode() -> Handlebars<'static> {
    let mut reg = Handlebars::new();
    // enable dev mode for template reloading
    reg.set_dev_mode(true);
    // register a template from the file
    // modified the file after the server starts to see things changing
    reg.register_template_file("tpl", "./examples/dev_mode/template.hbs")
        .unwrap();

    reg
}

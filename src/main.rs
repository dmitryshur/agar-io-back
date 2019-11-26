extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;
extern crate rand;
extern crate serde;
extern crate serde_json;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};

mod actors;
mod client_messages;
mod server_messages;
mod consts;
mod utils;

use actors::{ws};

fn index(request: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    actix_web_actors::ws::start(ws::Ws::default(), &request, stream)
}

pub struct Data {
    pub age: i32,
}

fn main() {
    println!("Running on 127.0.0.1:5555");
    HttpServer::new(|| {
        App::new()
            .route("/ws/", web::get().to(index))
    })
    .bind("127.0.0.1:5555")
    .unwrap()
    .run()
    .unwrap();
}

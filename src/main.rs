extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;
extern crate rand;
extern crate serde;
extern crate serde_json;

use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};

mod actors;
mod client_messages;
mod consts;
mod server_messages;
mod utils;

use actors::{world, ws};

fn index(
    request: HttpRequest,
    stream: web::Payload,
    world_actor: web::Data<Addr<world::World>>,
) -> Result<HttpResponse, Error> {
    actix_web_actors::ws::start(ws::Ws::new(world_actor.get_ref().clone()), &request, stream)
}

pub struct Data {
    pub age: i32,
}

fn main() -> std::io::Result<()> {
    println!("Running on 127.0.0.1:5555");
    let system = System::new("agar-io");
    let world_actor = world::World::default().start();

    HttpServer::new(move || {
        App::new()
            .data(world_actor.clone())
            .route("/ws/", web::get().to(index))
    })
    .bind("127.0.0.1:5555")?
    .start();

    system.run()
}

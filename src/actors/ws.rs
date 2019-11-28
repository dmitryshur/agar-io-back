use actix;
use actix::prelude::*;
use actix_web_actors::ws;
use serde_json;

use crate::actors::world;
use crate::client_messages::{ClientRequests};
use crate::server_messages::CreateResponse;

#[derive(Debug)]
pub struct Ws {
    world_actor: Addr<world::World>,
}

impl Ws {
    pub fn new(world_actor: Addr<world::World>) -> Self {
        Ws { world_actor }
    }
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<CreateResponse> for Ws {
    type Result = ();

    fn handle(&mut self, message: CreateResponse, context: &mut ws::WebsocketContext<Self>) {
        if let Ok(json) = serde_json::to_string(&message) {
            context.text(json);
        }
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, socket_message: ws::Message, context: &mut Self::Context) {
        if let ws::Message::Text(payload) = socket_message {
            let message: ClientRequests =
                serde_json::from_str(&payload).unwrap_or(ClientRequests::Invalid);

            match message {
                ClientRequests::Create(msg) => {
                    let create_request_future = self
                        .world_actor
                        .send(msg)
                        .into_actor(self)
                        .map(move |result, _actor, context| {
                            let result_json = serde_json::to_string(&result.unwrap())
                                .expect("Couldn't parse CreateResponse");
                            context.text(result_json);
                        })
                        .map_err(|error, _actor, _context| {
                            println!("{}", error);
                        });

                    context.spawn(create_request_future);
                }
                ClientRequests::Move(msg) => {
                    self.world_actor.do_send(msg);
                }
                ClientRequests::Win(_msg) => {
                    println!("Got win message");
                }
                ClientRequests::Lose(_msg) => {
                    println!("Got lose message");
                }
                ClientRequests::Invalid => {
                    println!("Got invalid message");
                }
            }
        }
    }
}

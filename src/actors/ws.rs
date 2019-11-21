use actix;
use actix::prelude::*;
use actix_web_actors::ws;
use serde_json;

use crate::actors::world;
use crate::client_messages::{ClientRequests};
use crate::server_messages::{CreateResponse};

#[derive(Debug)]
pub struct Ws {
    players_addresses: Vec<i32>,
    world_actor: Option<Addr<world::World>>,
}

impl Default for Ws {
    fn default() -> Self {
        Ws {
            players_addresses: vec![],
            world_actor: None,
        }
    }
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut ws::WebsocketContext<Self>) {
        self.world_actor = Some(world::World::new(context.address()).start());
    }
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
    fn handle(&mut self, socket_message: ws::Message, _context: &mut Self::Context) {
        if let ws::Message::Text(payload) = socket_message {
            let message: ClientRequests =
                serde_json::from_str(&payload).unwrap_or(ClientRequests::Invalid);

            match message {
                ClientRequests::Create(msg) => {
                    self.world_actor.as_ref().unwrap().do_send(msg);
                }
                ClientRequests::Move(_msg) => {}
                ClientRequests::Win(_msg) => {}
                ClientRequests::Lose(_msg) => {}
                ClientRequests::Invalid => {}
            }
        }
    }
}

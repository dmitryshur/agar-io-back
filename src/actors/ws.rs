use actix;
use actix::prelude::*;
use actix_web_actors::ws;
use serde_json;

use std::time::{Instant};

use crate::actors::{dots, world};
use crate::client_messages::{ClientRequests, CreateRequest, MoveRequest};
use crate::consts::{CLIENT_TIMEOUT, PING_SEND_INTERVAL};
use crate::server_messages;

// ********
// Messages
// ********
#[derive(Message)]
#[rtype(result = "Result<server_messages::CreateResponse, ()>")]
pub struct ConnectPlayer {
    pub request: CreateRequest,
    pub address: Addr<Ws>,
}

#[derive(Message)]
pub struct DisconnectPlayer {
    pub address: Addr<Ws>,
}

#[derive(Message)]
pub struct MovePlayer {
    pub request: MoveRequest,
    pub address: Addr<Ws>,
}

// ********
// Types
// ********
#[derive(Debug)]
pub struct Ws {
    world_actor: Addr<world::World>,
    ping_timestamp: Instant,
}

impl Ws {
    pub fn new(world_actor: Addr<world::World>) -> Self {
        Ws {
            world_actor,
            ping_timestamp: Instant::now(),
        }
    }
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        context.run_interval(PING_SEND_INTERVAL, |actor, context| {
            if Instant::now().duration_since(actor.ping_timestamp) > CLIENT_TIMEOUT {
                actor.world_actor.do_send(DisconnectPlayer {
                    address: context.address(),
                });

                context.stop();
                return;
            }

            //            context.ping("");
        });
    }
}

// ********
// Handlers
// ********
impl StreamHandler<ws::Message, ws::ProtocolError> for Ws {
    fn handle(&mut self, socket_message: ws::Message, context: &mut Self::Context) {
        match socket_message {
            ws::Message::Ping(payload) => {
                self.ping_timestamp = Instant::now();
                context.pong(&payload);
            }
            ws::Message::Pong(_payload) => {
                self.ping_timestamp = Instant::now();
            }
            ws::Message::Text(payload) => {
                self.ping_timestamp = Instant::now();
                let message: ClientRequests = serde_json::from_str(&payload).unwrap_or(ClientRequests::Invalid);

                match message {
                    ClientRequests::Create(msg) => {
                        let create_request_future = self
                            .world_actor
                            .send(ConnectPlayer {
                                request: msg,
                                address: context.address(),
                            })
                            .into_actor(self)
                            .map(move |result, _actor, context| {
                                let result_json =
                                    serde_json::to_string(&result.unwrap()).expect("Couldn't parse CreateResponse");
                                context.text(result_json);
                            })
                            .map_err(|error, _actor, _context| {
                                println!("{}", error);
                            });

                        context.spawn(create_request_future);
                    }
                    ClientRequests::Move(msg) => {
                        self.world_actor.do_send(MovePlayer {
                            request: msg,
                            address: context.address(),
                        });
                    }
                    ClientRequests::Invalid => {
                        println!("Invalid message");
                    }
                }
            }
            _ => (),
        }
    }
}

impl Handler<dots::GetDotsResult> for Ws {
    type Result = ();

    fn handle(&mut self, message: dots::GetDotsResult, context: &mut Self::Context) {
        let result_json = serde_json::to_string(&server_messages::DotsResponse { dots: message.dots })
            .expect("Couldn't parse DotsResponse");

        context.text(result_json);
    }
}

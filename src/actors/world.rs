use actix;
use actix::prelude::*;
use futures::future;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::Arc;

use crate::actors::dots::Dots;
use crate::actors::ws::Ws;
use crate::actors::{dots, players, ws};
use crate::consts::{DOTS_SEND_INTERVAL, PLAYERS_SEND_INTERVAL, WORLD_X_SIZE, WORLD_Y_SIZE};
use crate::server_messages;

// ********
// Types
// ********
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct World {
    players_connected: HashMap<Addr<Ws>, Uuid>,
    players_actor: Arc<Addr<players::Players>>,
    dots_actor: Arc<Addr<Dots>>,
}

impl World {
    fn run_dots_interval(&self, context: &mut Context<Self>) {
        context.run_interval(DOTS_SEND_INTERVAL, |actor, _context| {
            for (address, id) in actor.players_connected.iter() {
                let players_actor = actor.players_actor.clone();
                let dots_actor = actor.dots_actor.clone();

                let player_id = id.clone();
                let player_address = address.clone();

                let get_player_dots_future = players_actor
                    .send(players::GetPlayer(player_id))
                    .and_then(move |result: players::Player| {
                        dots_actor.send(dots::GetDots {
                            id: player_id,
                            coordinates: result.coordinates,
                            viewport_size: result.viewport_size,
                        })
                    })
                    .map(move |result| {
                        player_address.do_send(result);
                    })
                    .map_err(|error| {
                        println!("{}", error);
                    });

                Arbiter::spawn(get_player_dots_future);
            }
        });
    }

    fn run_players_interval(&self, context: &mut Context<Self>) {
        context.run_interval(PLAYERS_SEND_INTERVAL, |actor, _context| {
            for id in actor.players_connected.values() {
                let players_actor = actor.players_actor.clone();

                let get_players_in_viewport_future = players_actor
                    .send(players::GetPlayersInViewport(*id))
                    .map(|result: players::GetPlayersInViewportResult| {
                        println!("{:?}", result);
                    })
                    .map_err(|error| {
                        println!("{}", error);
                    });

                Arbiter::spawn(get_players_in_viewport_future);
            }
        });
    }
}

impl Actor for World {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        self.run_dots_interval(context);
        self.run_players_interval(context);
    }
}

impl Default for World {
    fn default() -> Self {
        World {
            players_connected: HashMap::new(),
            players_actor: Arc::new(players::Players::default().start()),
            dots_actor: Arc::new(Dots::default().start()),
        }
    }
}

// ********
// Handlers
// ********
impl Handler<ws::ConnectPlayer> for World {
    type Result = ResponseActFuture<Self, server_messages::CreateResponse, ()>;

    fn handle(&mut self, message: ws::ConnectPlayer, _context: &mut Context<Self>) -> Self::Result {
        let player_address = message.address.clone();
        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        let connect_player_future = players_actor
            .send(players::CreatePlayer(message.request.viewport_size))
            .and_then(move |new_player| {
                dots_actor.send(dots::GetDots {
                    id: new_player.id,
                    coordinates: new_player.coordinates,
                    viewport_size: message.request.viewport_size,
                })
            })
            .and_then(|result| {
                future::ok(server_messages::CreateResponse {
                    id: result.player_id,
                    world_size: Coordinates {
                        x: WORLD_X_SIZE,
                        y: WORLD_Y_SIZE,
                    },
                    dots: result.dots,
                })
            })
            .into_actor(self)
            .map(move |result, actor, _context| {
                actor.players_connected.insert(player_address, result.id);
                result
            })
            .map_err(|error, _actor, _context| {
                println!("{}", error);
            });

        Box::new(connect_player_future)
    }
}

impl Handler<ws::DisconnectPlayer> for World {
    type Result = ();

    fn handle(&mut self, message: ws::DisconnectPlayer, _context: &mut Context<Self>) {
        self.players_connected.remove(&message.address);
    }
}

impl Handler<ws::MovePlayer> for World {
    type Result = ();

    fn handle(&mut self, message: ws::MovePlayer, _context: &mut Context<Self>) {
        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        if message.request.dots_consumed.len() > 0 {
            dots_actor.do_send(dots::DeleteDots(message.request.dots_consumed));
        }

        players_actor.do_send(players::MovePlayer {
            id: message.request.id,
            size: message.request.size,
            moved: message.request.moved,
        });
    }
}

use actix;
use actix::prelude::*;
use futures::Future;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::Arc;

use crate::actors::dots::Dots;
use crate::actors::ws::Ws;
use crate::actors::{dots, players, ws};
use crate::consts::{WORLD_X_SIZE, WORLD_Y_SIZE};
use crate::server_messages;

// ********
// Types
// ********
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct World {
    players_connected: HashMap<Addr<Ws>, Uuid>,
    viewport_size: Coordinates,
    players_actor: Arc<Addr<players::Players>>,
    dots_actor: Arc<Addr<Dots>>,
}

impl Actor for World {
    type Context = Context<Self>;
}

impl Default for World {
    fn default() -> Self {
        World {
            players_connected: HashMap::new(),
            viewport_size: Coordinates { x: 0, y: 0 },
            players_actor: Arc::new(players::Players::default().start()),
            dots_actor: Arc::new(Dots::default().start()),
        }
    }
}

impl Handler<ws::ConnectPlayer> for World {
    type Result = Box<dyn Future<Item = server_messages::CreateResponse, Error = ()>>;

    fn handle(&mut self, message: ws::ConnectPlayer, _context: &mut Context<Self>) -> Self::Result {
        self.viewport_size = message.request.viewport_size;

        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        let connect_player_future = players_actor
            .send(players::CreatePlayer)
            .and_then(move |new_player| {
                dots_actor
                    .send(dots::GetDots {
                        coordinates: new_player.coordinates,
                        viewport_size: message.request.viewport_size,
                    })
                    .map(move |value| server_messages::CreateResponse {
                        id: new_player.id,
                        world_size: Coordinates {
                            x: WORLD_X_SIZE,
                            y: WORLD_Y_SIZE,
                        },
                        dots: value.dots,
                    })
            })
            .map_err(|error| {
                println!("{}", error);
            });

        Box::new(connect_player_future)
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

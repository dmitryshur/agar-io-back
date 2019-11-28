use actix;
use actix::prelude::*;
use futures::Future;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::Arc;

use crate::actors::dots::{Dots, DotsCreateAnswer};
use crate::actors::players::{PlayerCreateAnswer, Players};
use crate::client_messages::{CreateRequest, LoseRequest, MoveRequest, WinRequest};
use crate::consts::{WORLD_X_SIZE, WORLD_Y_SIZE};
use crate::server_messages::CreateResponse;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

pub struct PlayerCreateMessage;

impl Message for PlayerCreateMessage {
    type Result = PlayerCreateAnswer;
}

pub struct GetDotsMessage {
    pub coordinates: Coordinates,
    pub viewport_size: Coordinates,
}

impl Message for GetDotsMessage {
    type Result = DotsCreateAnswer;
}

pub struct DeleteDotsMessage(pub Vec<Uuid>);

impl Message for DeleteDotsMessage {
    type Result = ();
}

pub struct MoveMessage {
    pub id: Uuid,
    pub moved: Coordinates,
    pub size: u32,
}

impl Message for MoveMessage {
    type Result = ();
}

#[derive(Debug)]
pub struct World {
    viewport_size: Coordinates,
    players_actor: Arc<Addr<Players>>,
    dots_actor: Arc<Addr<Dots>>,
}

impl World {
    #[allow(dead_code)]
    fn handle_win_message(&self, _message: WinRequest) {}

    #[allow(dead_code)]
    fn handle_lose_message(&self, _message: LoseRequest) {}
}

impl Actor for World {
    type Context = Context<Self>;
}

impl Default for World {
    fn default() -> Self {
        World {
            viewport_size: Coordinates { x: 0, y: 0 },
            players_actor: Arc::new(Players::default().start()),
            dots_actor: Arc::new(Dots::default().start()),
        }
    }
}

impl Handler<CreateRequest> for World {
    type Result = Box<dyn Future<Item = CreateResponse, Error = ()>>;

    fn handle(&mut self, message: CreateRequest, _context: &mut Context<Self>) -> Self::Result {
        self.viewport_size = message.viewport_size;

        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        let create_future = players_actor
            .send(PlayerCreateMessage)
            .and_then(move |new_player| {
                dots_actor
                    .send(GetDotsMessage {
                        coordinates: new_player.coordinates,
                        viewport_size: message.viewport_size,
                    })
                    .map(move |value| CreateResponse {
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

        Box::new(create_future)
    }
}

impl Handler<MoveRequest> for World {
    type Result = ();

    fn handle(&mut self, message: MoveRequest, _context: &mut Context<Self>) {
        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        if message.dots_consumed.len() > 0 {
            dots_actor.do_send(DeleteDotsMessage(message.dots_consumed));
        }

        players_actor.do_send(MoveMessage {
            id: message.id,
            size: message.size,
            moved: message.moved,
        });
    }
}

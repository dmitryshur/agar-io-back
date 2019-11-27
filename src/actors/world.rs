use actix;
use actix::prelude::*;
use futures::Future;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::actors::dots::{Dots, DotsCreateResponse};
use crate::actors::players::{PlayerCreateResponse, Players};
use crate::actors::ws::{ActorClientCreate, Ws};
use crate::client_messages::{CreateRequest, LoseRequest, MoveRequest, WinRequest};
use crate::consts::{WORLD_X_SIZE, WORLD_Y_SIZE};
use crate::server_messages::CreateResponse;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

pub struct ActorPlayerCreateRequest;

impl Message for ActorPlayerCreateRequest {
    type Result = PlayerCreateResponse;
}

pub struct ActorGetDotsRequest {
    pub coordinates: Coordinates,
    pub viewport_size: Coordinates,
}

impl Message for ActorGetDotsRequest {
    type Result = DotsCreateResponse;
}

pub struct ActorDeleteDots(pub Vec<Uuid>);

impl Message for ActorDeleteDots {
    type Result = ();
}

pub struct ActorMovePlayer {
    pub id: Uuid,
    pub moved: Coordinates,
    pub size: u32,
}

impl Message for ActorMovePlayer {
    type Result = ();
}

#[derive(Debug)]
pub struct World {
    viewport_size: Coordinates,
    players_actor: Arc<Addr<Players>>,
    dots_actor: Arc<Addr<Dots>>,
}

impl World {
    fn handle_create_message(
        &mut self,
        message: ActorClientCreate,
    ) -> impl Future<Item = CreateResponse, Error = ()> {
        self.viewport_size = message.data.viewport_size;

        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        let create_future = players_actor
            .send(ActorPlayerCreateRequest)
            .and_then(move |new_player| {
                dots_actor
                    .send(ActorGetDotsRequest {
                        coordinates: new_player.coordinates,
                        viewport_size: message.data.viewport_size,
                    })
                    .map(move |value| {
                        println!("test");

                        CreateResponse {
                            id: new_player.id,
                            world_size: Coordinates {
                                x: WORLD_X_SIZE,
                                y: WORLD_Y_SIZE,
                            },
                            dots: value.dots,
                        }
                    })
            })
            .map_err(|_error| ());

        create_future
    }

    #[allow(dead_code)]
    // fn handle_move_message(&self, message: MoveRequest) {
    //     let players_actor = self.players_actor.clone();
    //     let dots_actor = self.dots_actor.clone();

    //     if message.dots_consumed.len() > 0 {
    //         dots_actor.do_send(ActorDeleteDots(message.dots_consumed));
    //     }

    //     players_actor.do_send(ActorMovePlayer {
    //         id: message.id,
    //         size: message.size,
    //         moved: message.moved,
    //     });
    // }
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

impl Handler<ActorClientCreate> for World {
    type Result = ();

    fn handle(&mut self, message: ActorClientCreate, _context: &mut Context<Self>) {
        let address = message.address.clone();

        let response_future = self
            .handle_create_message(message)
            .map(move |response| {
                address.do_send(response);
            })
            .map_err(|error| {
                println!("{:?}", error);
            });

        Arbiter::spawn(response_future);
    }
}

// impl Handler<MoveRequest> for World {
//     type Result = ();

//     fn handle(&mut self, message: MoveRequest, _context: &mut Context<Self>) {
//         self.handle_move_message(message);
//     }
// }

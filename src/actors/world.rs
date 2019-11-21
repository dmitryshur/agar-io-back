use actix;
use actix::prelude::*;
use futures::{Future};
use rand::Rng;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::actors::dots::{Dots};
use crate::actors::players::Players;
use crate::actors::ws::Ws;
use crate::client_messages::{CreateRequest, LoseRequest, MoveRequest, WinRequest};
use crate::consts::{WORLD_X_SIZE, WORLD_Y_SIZE};
use crate::server_messages::CreateResponse;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u32,
    pub y: u32,
}

pub struct PlayerCreateRequest;
pub struct DotsCreateRequest {
    pub coordinates: Coordinates,
    pub viewport_size: Coordinates,
}

#[derive(Debug)]
pub struct World {
    viewport_size: Coordinates,
    ws_actor: Arc<Addr<Ws>>,
    players_actor: Arc<Addr<Players>>,
    dots_actor: Arc<Addr<Dots>>,
}

impl World {
    pub fn new(address: Addr<Ws>) -> Self {
        World {
            viewport_size: Coordinates { x: 0, y: 0 },
            ws_actor: Arc::new(address),
            players_actor: Arc::new(Players::default().start()),
            dots_actor: Arc::new(Dots::default().start()),
        }
    }

    fn handle_create_message(
        &mut self,
        message: CreateRequest,
    ) -> impl Future<Item = CreateResponse, Error = ()> {
        self.viewport_size = message.viewport_size;

        let players_actor = self.players_actor.clone();
        let dots_actor = self.dots_actor.clone();

        let create_future = players_actor
            .send(PlayerCreateRequest)
            .and_then(move |new_player| {
                dots_actor
                    .send(DotsCreateRequest {
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
            .map_err(|_error| ());

        create_future
    }

    #[allow(dead_code)]
    fn handle_move_message(&self, _message: MoveRequest) {
        println!("Got move message");
    }

    #[allow(dead_code)]
    fn handle_win_message(&self, _message: WinRequest) {}

    #[allow(dead_code)]
    fn handle_lose_message(&self, _message: LoseRequest) {}
}

impl Actor for World {
    type Context = Context<Self>;
}

impl Handler<CreateRequest> for World {
    type Result = ();

    fn handle(&mut self, message: CreateRequest, _context: &mut Context<Self>) {
        let ws_actor = self.ws_actor.clone();

        let response_future = self
            .handle_create_message(message)
            .map(move |response| {
                ws_actor.clone().do_send(response);
            })
            .map_err(|error| {
                println!("{:?}", error);
            });

        Arbiter::spawn(response_future);
    }
}

pub fn generate_coordinates() -> Coordinates {
    let mut generator = rand::thread_rng();
    let x: u32 = generator.gen_range(0, WORLD_X_SIZE);
    let y: u32 = generator.gen_range(0, WORLD_Y_SIZE);

    Coordinates { x, y }
}

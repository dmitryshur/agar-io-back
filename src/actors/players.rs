use std::collections::HashMap;

use actix::dev::{MessageResponse, ResponseChannel};
use actix::{Actor, Context, Handler, Message};
use uuid::Uuid;

use crate::actors::world::{generate_coordinates, Coordinates, PlayerCreateRequest};
use crate::consts::{DEFAULT_PLAYER_SIZE};

#[derive(Debug, Copy, Clone)]
pub struct PlayerCreateResponse {
    pub id: Uuid,
    pub coordinates: Coordinates,
}

#[derive(Clone, Debug)]
struct Player {
    pub size: u32,
    pub coordinates: Coordinates,
}

#[derive(Debug)]
pub struct Players {
    players: HashMap<Uuid, Player>,
    players_count: u32,
}

impl Default for Players {
    fn default() -> Self {
        Players {
            players: HashMap::new(),
            players_count: 0,
        }
    }
}

impl Actor for Players {
    type Context = Context<Self>;
}

impl Message for PlayerCreateRequest {
    type Result = PlayerCreateResponse;
}

impl<A, M> MessageResponse<A, M> for PlayerCreateResponse
where
    A: Actor,
    M: Message<Result = PlayerCreateResponse>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

impl Handler<PlayerCreateRequest> for Players {
    type Result = PlayerCreateResponse;

    fn handle(
        &mut self,
        _message: PlayerCreateRequest,
        _context: &mut Context<Self>,
    ) -> Self::Result {
        let new_player_coordinates = generate_coordinates();
        let new_player = Player {
            size: DEFAULT_PLAYER_SIZE,
            coordinates: new_player_coordinates,
        };
        let player_id = Uuid::new_v4();

        self.players.insert(player_id, new_player);
        self.players_count += 1;

        PlayerCreateResponse {
            id: player_id,
            coordinates: new_player_coordinates.clone(),
        }
    }
}

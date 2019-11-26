use std::collections::HashMap;

use actix::dev::{MessageResponse, ResponseChannel};
use actix::{Actor, Context, Handler, Message};
use uuid::Uuid;

use crate::actors::world::{ActorMovePlayer, ActorPlayerCreateRequest, Coordinates};
use crate::consts::DEFAULT_PLAYER_SIZE;
use crate::utils::{generate_coordinates};

#[derive(Debug, Copy, Clone)]
pub struct PlayerCreateResponse {
    pub id: Uuid,
    pub coordinates: Coordinates,
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub size: u32,
    pub coordinates: Coordinates,
}

#[derive(Debug, Clone)]
pub struct Players {
    pub players: HashMap<Uuid, Player>,
    pub players_count: u32,
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

impl Handler<ActorPlayerCreateRequest> for Players {
    type Result = PlayerCreateResponse;

    fn handle(
        &mut self,
        _message: ActorPlayerCreateRequest,
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

impl Handler<ActorMovePlayer> for Players {
    type Result = ();

    fn handle(&mut self, message: ActorMovePlayer, _context: &mut Context<Self>) {
        if let Some(player) = self.players.get_mut(&message.id) {
            player.size = message.size;
            player.coordinates.x += message.moved.x;
            player.coordinates.y += message.moved.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::prelude::*;
    use futures::Future;
    use std::sync::Arc;

    struct GetState;

    impl Message for GetState {
        type Result = Players;
    }

    impl<A, M> MessageResponse<A, M> for Players
    where
        A: Actor,
        M: Message<Result = Players>,
    {
        fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
            if let Some(tx) = tx {
                tx.send(self);
            }
        }
    }

    impl Handler<GetState> for Players {
        type Result = Players;

        fn handle(&mut self, _message: GetState, _context: &mut Context<Self>) -> Players {
            let players = self.players.clone();

            Players {
                players,
                players_count: self.players_count,
            }
        }
    }

    #[test]
    fn test_players_actor_creation() {
        let mut system = System::new("players_creation");
        let player_actor = Arc::new(Players::default().start());
        let result_future = player_actor
            .clone()
            .send(ActorPlayerCreateRequest)
            .and_then(|_result| {
                player_actor.clone().send(GetState).map(|result| {
                    assert_eq!(result.players_count, 1);
                    assert_eq!(result.players.len(), 1);
                })
            })
            .and_then(|_res| {
                player_actor
                    .clone()
                    .send(ActorPlayerCreateRequest)
                    .and_then(|_result| {
                        player_actor.clone().send(GetState).map(|result| {
                            assert_eq!(result.players_count, 2);
                            assert_eq!(result.players.len(), 2);
                        })
                    })
            })
            .map_err(|error| {
                println!("{:?}", error);
            });

        system.block_on(result_future).expect("System  error");
    }
}

use std::collections::HashMap;

use actix::dev::{MessageResponse};
use actix::prelude::*;
use uuid::Uuid;

use crate::actors::world::{Coordinates};
use crate::consts::DEFAULT_PLAYER_SIZE;
use crate::utils::generate_coordinates;

// ********
// Messages
// ********
#[derive(Message)]
#[rtype(result = "CreatePlayerResult")]
pub struct CreatePlayer;

#[derive(Message)]
pub struct MovePlayer {
    pub id: Uuid,
    pub moved: Coordinates,
    pub size: u32,
}

#[allow(dead_code)]
pub struct GetPlayer;

// ****************
// Messages results
// ****************
#[derive(MessageResponse, Copy, Clone)]
pub struct CreatePlayerResult {
    pub id: Uuid,
    pub coordinates: Coordinates,
}

// ********
// Types
// ********
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub size: u32,
    pub coordinates: Coordinates,
}

#[derive(MessageResponse, Debug, Clone)]
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

impl Handler<CreatePlayer> for Players {
    type Result = CreatePlayerResult;

    fn handle(&mut self, _message: CreatePlayer, _context: &mut Context<Self>) -> Self::Result {
        let new_player_coordinates = generate_coordinates();
        let new_player = Player {
            size: DEFAULT_PLAYER_SIZE,
            coordinates: new_player_coordinates,
        };
        let player_id = Uuid::new_v4();

        self.players.insert(player_id, new_player);
        self.players_count += 1;

        CreatePlayerResult {
            id: player_id,
            coordinates: new_player_coordinates.clone(),
        }
    }
}

impl Handler<MovePlayer> for Players {
    type Result = ();

    fn handle(&mut self, message: MovePlayer, _context: &mut Context<Self>) {
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
    use futures::Future;
    use std::sync::Arc;

    #[derive(Message)]
    #[rtype(result = "Players")]
    struct GetState;

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
            .send(CreatePlayer)
            .and_then(|_result| {
                player_actor.clone().send(GetState).map(|result| {
                    assert_eq!(result.players_count, 1);
                    assert_eq!(result.players.len(), 1);
                })
            })
            .and_then(|_res| {
                player_actor.clone().send(CreatePlayer).and_then(|_result| {
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

use std::collections::HashMap;

use actix::dev::MessageResponse;
use actix::prelude::*;
use uuid::Uuid;

use crate::actors::world::Coordinates;
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

#[derive(Debug, Message)]
#[rtype(result = "Player")]
pub struct GetPlayer(pub Uuid);

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
#[derive(Clone, Copy, Debug, MessageResponse)]
pub struct Player {
    pub size: u32,
    pub coordinates: Coordinates,
}

#[derive(MessageResponse, Debug, Clone)]
pub struct Players {
    pub players: HashMap<Uuid, Player>,
    pub players_count: u32,
}

#[cfg(test)]
impl Players {
    fn new(players: HashMap<Uuid, Player>, players_count: u32) -> Self {
        Players { players, players_count }
    }
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

impl Handler<GetPlayer> for Players {
    type Result = Player;

    fn handle(&mut self, message: GetPlayer, _context: &mut Context<Self>) -> Self::Result {
        let matching_player = self.players.get(&message.0).unwrap();

        matching_player.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future, Future};
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
    fn test_players_actor_create() {
        let mut system = System::new("players_creation");
        let player_actor = Arc::new(Players::default().start());

        let create_player_future = player_actor
            .send(GetState)
            .and_then(|result| {
                let new_hashmap: HashMap<Uuid, Player> = HashMap::new();

                assert_eq!(result.players.len(), new_hashmap.len());
                assert_eq!(result.players_count, 0);
                future::ok(())
            })
            .and_then(|_future| player_actor.send(CreatePlayer))
            .and_then(|result| {
                assert_eq!(result.coordinates, Coordinates { x: 100, y: 100 });
                player_actor.send(GetState)
            })
            .and_then(|result| {
                assert_eq!(result.players_count, 1);
                player_actor.send(CreatePlayer)
            })
            .and_then(|_result| player_actor.send(GetState))
            .map(|result| {
                assert_eq!(result.players_count, 2);
                assert_eq!(result.players.len(), 2);
            });

        system.block_on(create_player_future).expect("System  error");
    }

    #[test]
    fn test_players_actor_move() {
        let mut system = System::new("players_moving");

        let mut initial_players = HashMap::new();
        let first_player_id =
            Uuid::parse_str("f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4").expect("Couldn't parse first player id");
        let second_player_id =
            Uuid::parse_str("78a40100-4dc3-46e4-8a91-00e0316586e4").expect("Couldn't parse second player id");

        initial_players.insert(
            first_player_id,
            Player {
                size: 10,
                coordinates: Coordinates { x: 200, y: 200 },
            },
        );
        initial_players.insert(
            second_player_id,
            Player {
                size: 20,
                coordinates: Coordinates { x: 250, y: 250 },
            },
        );

        let player_actor = Arc::new(Players::new(initial_players, 2).start());

        player_actor.do_send(MovePlayer {
            id: first_player_id,
            moved: Coordinates { x: 10, y: 10 },
            size: 10,
        });

        player_actor.do_send(MovePlayer {
            id: second_player_id,
            moved: Coordinates { x: 50, y: 40 },
            size: 15,
        });

        let move_player_future = player_actor.send(GetState).map(|result| {
            assert_eq!(result.players.len(), 2);
            assert_eq!(result.players_count, 2);

            let first_player = result.players.get(&first_player_id).unwrap();
            let second_player = result.players.get(&second_player_id).unwrap();

            assert_eq!(first_player.size, 10);
            assert_eq!(first_player.coordinates, Coordinates { x: 210, y: 210 });

            assert_eq!(second_player.size, 15);
            assert_eq!(second_player.coordinates, Coordinates { x: 300, y: 290 });
        });

        system.block_on(move_player_future).expect("System  error");
    }

    #[test]
    fn test_players_actor_win() {}

    #[test]
    fn test_players_actor_lose() {}
}

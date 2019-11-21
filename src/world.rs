use crate::consts::*;
use crate::player::Player;
use actix::{Actor, StreamHandler, AsyncContext, Message};
use actix_web_actors::ws;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::messages::{CreationMessage, Messages};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    x: u32,
    y: u32,
}

#[derive(Debug)]
pub struct World {
    players: HashMap<Uuid, Player>,
    players_count: u32,
    dots: HashMap<Uuid, Coordinates>,
    viewport_size: Coordinates,
}

impl World {
    fn find_viewport_dots(&self, player: &Player) -> HashMap<Uuid, Coordinates> {
        let min_x = player.coordinates.x - (self.viewport_size.x / 2) - DELTA_VIEWPORT;
        let max_x = player.coordinates.x + (self.viewport_size.x / 2) + DELTA_VIEWPORT;
        let min_y = player.coordinates.y - (self.viewport_size.y / 2) - DELTA_VIEWPORT;
        let max_y = player.coordinates.y + (self.viewport_size.y / 2) + DELTA_VIEWPORT;

        let dots_in_viewport: HashMap<Uuid, Coordinates> = self
            .dots
            .iter()
            .filter(|(_id, coordinates)| {
                coordinates.x > min_x
                    && (coordinates.x + DOT_SIZE < max_x)
                    && coordinates.y > min_y
                    && coordinates.y + DOT_SIZE < max_y
            })
            .map(|(id, coordinates)| {
                (
                    *id,
                    Coordinates {
                        x: coordinates.x - min_x,
                        y: coordinates.y - min_y,
                    },
                )
            })
            .collect();

        dots_in_viewport
    }
}

impl Default for World {
    fn default() -> Self {
        World {
            players: HashMap::new(),
            players_count: 0,
            dots: generate_dots(),
            viewport_size: Coordinates { x: 0, y: 0 },
        }
    }
}

impl Actor for World {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for World {
    fn handle(&mut self, socket_message: ws::Message, context: &mut Self::Context) {
        if let ws::Message::Text(payload) = socket_message {
            let player_message: Messages =
                serde_json::from_str(&payload).unwrap_or(Messages::Invalid);

            match player_message {
                Messages::Action(message) => {
                    if let "create" = message.action.as_ref() {
                        let new_coordinates = generate_coordinates(WORLD_X_SIZE, WORLD_Y_SIZE);
                        let new_player = Player {
                            size: DEFAULT_PLAYER_SIZE,
                            coordinates: new_coordinates.clone(),
                        };

                        // The viewport size is based on the client window size
                        self.viewport_size =
                            message.viewport_size.unwrap_or(Coordinates { x: 0, y: 0 });
                        let dots = self.find_viewport_dots(&new_player);
                        let player_id = Uuid::new_v4();
                        self.players.insert(player_id, new_player.clone());
                        self.players_count += 1;

                        let creation_message = serde_json::to_string(&CreationMessage::new(
                            player_id,
                            new_player.size,
                            Coordinates {
                                x: WORLD_X_SIZE,
                                y: WORLD_Y_SIZE,
                            },
                            new_coordinates,
                            dots,
                        ))
                        .unwrap();

                        context.text(creation_message)
                    }
                }
                Messages::Move(message) => {
                    let matching_player = self.players.get(&message.id);
                    if let Some(player) = matching_player {
                        context.text(format!("{:?}", player));
                    }
                }
                Messages::Invalid => {}
            }
        }
    }
}

fn generate_coordinates(max_width: u32, max_height: u32) -> Coordinates {
    let mut generator = rand::thread_rng();
    let x: u32 = generator.gen_range(0, max_width);
    let y: u32 = generator.gen_range(0, max_height);

    Coordinates { x, y }
}

fn generate_dots() -> HashMap<Uuid, Coordinates> {
    let dots: HashMap<Uuid, Coordinates> = (0..MAX_DOTS_AMOUNT)
        .map(|_| {
            (
                Uuid::new_v4(),
                generate_coordinates(WORLD_X_SIZE, WORLD_Y_SIZE),
            )
        })
        .collect();

    dots
}

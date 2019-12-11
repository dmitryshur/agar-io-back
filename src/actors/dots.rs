use actix::dev::MessageResponse;
use actix::prelude::*;
use uuid::Uuid;

use std::collections::HashMap;

use crate::actors::world::Coordinates;
use crate::consts::{DELTA_VIEWPORT, DOTS_CREATE_INTERVAL, DOT_SIZE, MAX_DOTS_AMOUNT};
use crate::utils::{generate_dots};

// ********
// Messages
// ********
#[derive(Message)]
#[rtype(result = "GetDotsResult")]
pub struct GetDots {
    pub id: Uuid,
    pub coordinates: Coordinates,
    pub viewport_size: Coordinates,
}

#[derive(Message)]
pub struct DeleteDots(pub Vec<Uuid>);

// ****************
// Messages results
// ****************
#[derive(MessageResponse, Message, Debug)]
pub struct GetDotsResult {
    pub dots: HashMap<Uuid, Coordinates>,
    pub player_id: Uuid,
}

// ********
// Types
// ********
#[derive(MessageResponse, Clone, Debug)]
pub struct Dots {
    pub dots: HashMap<Uuid, Coordinates>,
    pub dots_count: u32,
    pub max_dots_amount: u32,
}

impl Dots {
    fn run_dots_creation_interval(&self, context: &mut Context<Self>) {
        context.run_interval(DOTS_CREATE_INTERVAL, |actor, _context| {
            if actor.dots_count < actor.max_dots_amount {
                generate_dots(&mut actor.dots, actor.max_dots_amount);
                actor.dots_count = actor.dots.len() as u32;
            }
        });
    }

    fn find_viewport_dots(&self, viewport_size: Coordinates, player: Coordinates) -> HashMap<Uuid, Coordinates> {
        let min_x = (player.x)
            .checked_sub((viewport_size.x / 2) - DELTA_VIEWPORT)
            .unwrap_or(0);
        let max_x = player.x + (viewport_size.x / 2) + DELTA_VIEWPORT;
        let min_y = (player.y)
            .checked_sub((viewport_size.y / 2) - DELTA_VIEWPORT)
            .unwrap_or(0);
        let max_y = player.y + (viewport_size.y / 2) + DELTA_VIEWPORT;

        let dots_in_viewport: HashMap<Uuid, Coordinates> = self
            .dots
            .iter()
            .filter(|(_id, coordinates)| {
                coordinates.x >= min_x
                    && (coordinates.x + DOT_SIZE < max_x)
                    && coordinates.y >= min_y
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

impl Default for Dots {
    fn default() -> Self {
        Dots {
            dots: HashMap::new(),
            dots_count: 0,
            max_dots_amount: MAX_DOTS_AMOUNT,
        }
    }
}

impl Handler<GetDots> for Dots {
    type Result = GetDotsResult;

    fn handle(&mut self, message: GetDots, _context: &mut Context<Self>) -> Self::Result {
        let dots = self.find_viewport_dots(message.viewport_size, message.coordinates);

        GetDotsResult {
            dots,
            player_id: message.id,
        }
    }
}

impl Handler<DeleteDots> for Dots {
    type Result = ();

    fn handle(&mut self, message: DeleteDots, _context: &mut Context<Self>) {
        for id in message.0 {
            self.dots.remove(&id);
            self.dots_count -= 1;
        }
    }
}

impl Actor for Dots {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Context<Self>) {
        generate_dots(&mut self.dots, self.max_dots_amount);
        self.dots_count = self.dots.len() as u32;
        self.run_dots_creation_interval(context);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{Future};
    use std::sync::Arc;

    #[derive(Message)]
    #[rtype(result = "Dots")]
    struct GetState;

    impl Handler<GetState> for Dots {
        type Result = Dots;

        fn handle(&mut self, _message: GetState, _context: &mut Context<Self>) -> Dots {
            let dots = self.dots.clone();

            Dots {
                dots,
                dots_count: self.dots_count,
                max_dots_amount: self.max_dots_amount,
            }
        }
    }

    #[test]
    fn test_dots_actor_get_dots() {
        let mut system = System::new("dots_creation");
        let dots_actor = Arc::new(Dots::default().start());

        let get_dots_future = dots_actor
            .send(GetState)
            .and_then(|result: Dots| {
                assert_eq!(result.dots.len(), 12);
                assert_eq!(result.dots_count, 12);
                assert_eq!(result.max_dots_amount, MAX_DOTS_AMOUNT);

                dots_actor.send(GetDots {
                    id: Uuid::parse_str("78a40100-4dc3-46e4-8a91-00e0316586e4").unwrap(),
                    coordinates: Coordinates { x: 0, y: 0 },
                    viewport_size: Coordinates { x: 1000, y: 1000 },
                })
            })
            .and_then(|result: GetDotsResult| {
                let dots_id = vec![
                    "f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4",
                    "e0183a5f-92af-4379-8d8d-cfd729d77d59",
                    "20066e7c-5dec-434f-97d1-663de407b05e",
                    "a0e3c51b-23a5-4809-b635-3eb6b3b1f794",
                    "77d40cd1-be99-44d2-9bcf-7450f736fdba",
                ];

                assert_eq!(result.dots.len(), 5);
                for id in dots_id {
                    assert_eq!(result.dots.contains_key(&Uuid::parse_str(id).unwrap()), true);
                }
                dots_actor.send(GetDots {
                    id: Uuid::parse_str("78a40100-4dc3-46e4-8a91-00e0316586e4").unwrap(),
                    coordinates: Coordinates { x: 1000, y: 1000 },
                    viewport_size: Coordinates { x: 1000, y: 1000 },
                })
            })
            .and_then(|result: GetDotsResult| {
                let dots_id = vec!["9bea8e0c-5d0a-4018-be7d-2ae9af088a0c"];

                assert_eq!(result.dots.len(), 1);
                for id in dots_id {
                    assert_eq!(result.dots.contains_key(&Uuid::parse_str(id).unwrap()), true);
                }

                dots_actor.send(GetDots {
                    id: Uuid::parse_str("78a40100-4dc3-46e4-8a91-00e0316586e4").unwrap(),
                    coordinates: Coordinates { x: 0, y: 600 },
                    viewport_size: Coordinates { x: 1000, y: 1000 },
                })
            })
            .map(|result: GetDotsResult| {
                let dots_id = vec![
                    "77d40cd1-be99-44d2-9bcf-7450f736fdba",
                    "be196b9b-6a85-4ba3-b7ac-c1dd02d6178a",
                    "018f87db-b89d-40f1-ab21-c1ba584fbca3",
                    "ffe016bf-a99e-470f-aaab-1c5f1eb1c04b",
                ];

                assert_eq!(result.dots.len(), 4);
                for id in dots_id {
                    assert_eq!(result.dots.contains_key(&Uuid::parse_str(id).unwrap()), true);
                }
            });

        system.block_on(get_dots_future).expect("System error");
    }

    #[test]
    fn test_dots_actor_delete_dots() {
        let mut system = System::new("dots_deletion");
        let dots_actor = Arc::new(Dots::default().start());

        let delete_dots_future = dots_actor
            .send(GetState)
            .and_then(|result: Dots| {
                assert_eq!(result.dots.len(), 12);
                assert_eq!(result.dots_count, 12);
                assert_eq!(result.max_dots_amount, MAX_DOTS_AMOUNT);

                dots_actor.do_send(DeleteDots(vec![
                    Uuid::parse_str("f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4").unwrap(),
                    Uuid::parse_str("e0183a5f-92af-4379-8d8d-cfd729d77d59").unwrap(),
                    Uuid::parse_str("20066e7c-5dec-434f-97d1-663de407b05e").unwrap(),
                    Uuid::parse_str("a0e3c51b-23a5-4809-b635-3eb6b3b1f794").unwrap(),
                    Uuid::parse_str("77d40cd1-be99-44d2-9bcf-7450f736fdba").unwrap(),
                ]));
                dots_actor.send(GetState)
            })
            .map(|result: Dots| {
                assert_eq!(result.dots.len(), 7);
                assert_eq!(result.dots_count, 7);
            });

        system.block_on(delete_dots_future).expect("System error");
    }
}

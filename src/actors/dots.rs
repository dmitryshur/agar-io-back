use actix::dev::MessageResponse;
use actix::prelude::*;
use uuid::Uuid;

use std::collections::HashMap;

use crate::actors::world::Coordinates;
use crate::consts::{DELTA_VIEWPORT, DOT_SIZE, MAX_DOTS_AMOUNT};
use crate::utils::generate_coordinates;

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
}

impl Dots {
    pub fn generate_dots(&mut self) {
        self.dots = (0..MAX_DOTS_AMOUNT)
            .map(|_| (Uuid::new_v4(), generate_coordinates()))
            .collect();
        self.dots_count = MAX_DOTS_AMOUNT;
    }

    fn find_viewport_dots(
        &self,
        viewport_size: Coordinates,
        player: Coordinates,
    ) -> HashMap<Uuid, Coordinates> {
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

impl Default for Dots {
    fn default() -> Self {
        Dots {
            dots: HashMap::new(),
            dots_count: 0,
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

    fn started(&mut self, _context: &mut Context<Self>) {
        self.generate_dots();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future, Future};
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
            }
        }
    }

    #[test]
    fn test_dots_actor_creation() {
        let mut system = System::new("dots_creation");
        let dots_actor = Arc::new(Dots::default().start());

        let result_future = dots_actor
            .clone()
            .send(GetState)
            .and_then(|state| {
                assert_eq!(state.dots_count, MAX_DOTS_AMOUNT);
                assert_eq!(state.dots.len(), MAX_DOTS_AMOUNT as usize);
                future::ok(())
            })
            .and_then(|_fut| {
                let get_dots_request = GetDots {
                    coordinates: Coordinates { x: 200, y: 200 },
                    viewport_size: Coordinates { x: 800, y: 600 },
                };
                dots_actor.clone().send(get_dots_request).map(|result| {
                    assert_eq!(result.dots.len(), MAX_DOTS_AMOUNT as usize);
                })
            })
            .and_then(|_fut| {
                let get_dots_request = GetDots {
                    coordinates: Coordinates { x: 500, y: 500 },
                    viewport_size: Coordinates { x: 800, y: 600 },
                };
                dots_actor.clone().send(get_dots_request).map(|result| {
                    assert_eq!(result.dots.len(), 0 as usize);
                })
            })
            .and_then(|_fut| dots_actor.clone().send(GetState))
            .and_then(|state| {
                let dots: Vec<Uuid> = state
                    .dots
                    .iter()
                    .take(2)
                    .map(|(uuid, _coordinates)| *uuid)
                    .collect();
                dots_actor.do_send(DeleteDots(dots));
                future::ok(())
            })
            .and_then(|_fut| dots_actor.clone().send(GetState))
            .map(|state| {
                assert_eq!(state.dots_count, MAX_DOTS_AMOUNT - 2);
                assert_eq!(state.dots.len(), (MAX_DOTS_AMOUNT - 2) as usize);
            })
            .map_err(|error| {
                panic!(error);
            });

        system.block_on(result_future).expect("System error");
    }
}

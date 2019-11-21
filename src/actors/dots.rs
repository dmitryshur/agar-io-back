use actix::dev::{MessageResponse, ResponseChannel};
use actix::prelude::*;
use uuid::Uuid;

use std::collections::HashMap;

use crate::actors::world::{generate_coordinates, Coordinates, DotsCreateRequest};
use crate::consts::{DELTA_VIEWPORT, DOT_SIZE, MAX_DOTS_AMOUNT};

#[derive(Debug)]
pub struct DotsCreateResponse {
    pub dots: HashMap<Uuid, Coordinates>,
}

#[derive(Clone, Debug)]
pub struct Dots {
    pub dots: HashMap<Uuid, Coordinates>,
    pub count: u32,
}

impl Dots {
    pub fn generate_dots(&mut self) {
        self.dots = (0..MAX_DOTS_AMOUNT)
            .map(|_| (Uuid::new_v4(), generate_coordinates()))
            .collect();
    }

    fn find_viewport_dots(
        &self,
        viewport_size: Coordinates,
        player: Coordinates,
    ) -> HashMap<Uuid, Coordinates> {
        let min_x = player.x - (viewport_size.x / 2) - DELTA_VIEWPORT;
        let max_x = player.x + (viewport_size.x / 2) + DELTA_VIEWPORT;
        let min_y = player.y - (viewport_size.y / 2) - DELTA_VIEWPORT;
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
            count: 0,
        }
    }
}

impl Message for DotsCreateRequest {
    type Result = DotsCreateResponse;
}

impl<A, M> MessageResponse<A, M> for DotsCreateResponse
where
    A: Actor,
    M: Message<Result = DotsCreateResponse>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

impl Handler<DotsCreateRequest> for Dots {
    type Result = DotsCreateResponse;

    fn handle(&mut self, message: DotsCreateRequest, _context: &mut Context<Self>) -> Self::Result {
        let dots = self.find_viewport_dots(message.viewport_size, message.coordinates);

        DotsCreateResponse { dots }
    }
}

impl Actor for Dots {
    type Context = Context<Self>;

    fn started(&mut self, _context: &mut Context<Self>) {
        self.generate_dots();
    }
}

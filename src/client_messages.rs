use serde::Deserialize;
use uuid::Uuid;

use crate::actors::world::Coordinates;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ClientRequests {
    Create(CreateRequest),
    Move(MoveRequest),
    Invalid,
}

#[derive(Deserialize, Debug)]
pub struct CreateRequest {
    pub viewport_size: Coordinates,
}

#[derive(Deserialize, Debug)]
pub struct MoveRequest {
    pub id: Uuid,
    pub size: u32,
    pub moved: Coordinates,
    pub dots_consumed: Vec<Uuid>,
}

#[derive(Deserialize, Debug)]
pub struct Invalid;

use actix::prelude::*;
use serde::{Deserialize};
use uuid::Uuid;

use crate::actors::world::Coordinates;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ClientRequests {
    Create(CreateRequest),
    Move(MoveRequest),
    Win(WinRequest),
    Lose(LoseRequest),
    Invalid,
}

#[derive(Deserialize, Debug)]
pub struct CreateRequest {
    pub viewport_size: Coordinates,
}

impl Message for CreateRequest {
  type Result = ();
}

#[derive(Deserialize, Debug)]
pub struct MoveRequest {
    pub id: Uuid,
    pub coordinates: Coordinates,
}

#[derive(Deserialize, Debug)]
pub struct WinRequest {
    win_id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct LoseRequest {
    lose_id: Uuid,
}

#[derive(Deserialize, Debug)]
pub struct Invalid;

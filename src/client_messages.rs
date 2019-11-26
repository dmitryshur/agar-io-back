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
    pub size: u32,
    pub moved: Coordinates,
    pub dots_consumed: Vec<Uuid>,
}

impl Message for MoveRequest {
  type Result = ();
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

use actix::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::world::Coordinates;

#[derive(Serialize)]
#[serde(untagged)]
pub enum ServerResponses {
    #[allow(dead_code)]
    Create(CreateResponse),
}

#[derive(Serialize, Debug)]
pub struct CreateResponse {
    pub id: Uuid,
    pub world_size: Coordinates,
    pub dots: HashMap<Uuid, Coordinates>,
}

impl Message for CreateResponse {
    type Result = ();
}

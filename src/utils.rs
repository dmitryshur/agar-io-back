#[cfg(not(test))]
use rand::Rng;

use crate::actors::world::Coordinates;

#[cfg(not(test))]
use crate::consts::{WORLD_X_SIZE, WORLD_Y_SIZE};

#[cfg(not(test))]
pub fn generate_coordinates() -> Coordinates {
  let mut generator = rand::thread_rng();
  let x: u32 = generator.gen_range(0, WORLD_X_SIZE);
  let y: u32 = generator.gen_range(0, WORLD_Y_SIZE);

  Coordinates { x, y }
}

#[cfg(test)]
pub fn generate_coordinates() -> Coordinates {
  Coordinates { x: 100, y: 100 }
}
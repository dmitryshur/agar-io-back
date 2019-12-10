#[cfg(not(test))]
use rand::Rng;
use uuid::Uuid;

use std::collections::HashMap;

use crate::actors::dots;
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

#[cfg(not(test))]
pub fn generate_dots(count: u32) -> HashMap<Uuid, Coordinates> {
    (0..count).map(|_| (Uuid::new_v4(), generate_coordinates())).collect()
}

#[cfg(test)]
pub fn generate_dots(count: u32) -> HashMap<Uuid, Coordinates> {
    vec![
        (
            Uuid::parse_str("f9168c5e-ceb2-4faa-b6bf-329bf39fa1e4").unwrap(),
            Coordinates { x: 0, y: 0 },
        ),
        (
            Uuid::parse_str("e0183a5f-92af-4379-8d8d-cfd729d77d59").unwrap(),
            Coordinates { x: 100, y: 0 },
        ),
        (
            Uuid::parse_str("20066e7c-5dec-434f-97d1-663de407b05e").unwrap(),
            Coordinates { x: 200, y: 0 },
        ),
        (
            Uuid::parse_str("a0e3c51b-23a5-4809-b635-3eb6b3b1f794").unwrap(),
            Coordinates { x: 0, y: 100 },
        ),
        (
            Uuid::parse_str("77d40cd1-be99-44d2-9bcf-7450f736fdba").unwrap(),
            Coordinates { x: 0, y: 200 },
        ),
        (
            Uuid::parse_str("be196b9b-6a85-4ba3-b7ac-c1dd02d6178a").unwrap(),
            Coordinates { x: 0, y: 900 },
        ),
        (
            Uuid::parse_str("018f87db-b89d-40f1-ab21-c1ba584fbca3").unwrap(),
            Coordinates { x: 100, y: 900 },
        ),
        (
            Uuid::parse_str("ffe016bf-a99e-470f-aaab-1c5f1eb1c04b").unwrap(),
            Coordinates { x: 200, y: 900 },
        ),
        (
            Uuid::parse_str("1f4c367c-f35f-4eda-8cb1-c4494fb542ab").unwrap(),
            Coordinates { x: 900, y: 0 },
        ),
        (
            Uuid::parse_str("1ff42309-6266-470a-9e4d-09babbc715f3").unwrap(),
            Coordinates { x: 900, y: 100 },
        ),
        (
            Uuid::parse_str("04679508-e52e-4038-8c45-9550e193265e").unwrap(),
            Coordinates { x: 900, y: 200 },
        ),
        (
            Uuid::parse_str("9bea8e0c-5d0a-4018-be7d-2ae9af088a0c").unwrap(),
            Coordinates { x: 900, y: 900 },
        ),
    ]
    .into_iter()
    .collect()
}

use std::time::Duration;

// World info
pub const WORLD_X_SIZE: u32 = 20_000;
pub const WORLD_Y_SIZE: u32 = 20_000;
pub const DELTA_VIEWPORT: u32 = 100;

// Player info
pub const DEFAULT_PLAYER_SIZE: u32 = 20;

// Dots info
pub const MAX_DOTS_AMOUNT: u32 = 10_000;
pub const DOT_SIZE: u32 = 10;

// Time
pub const DOTS_SEND_INTERVAL: Duration = Duration::from_secs(2);

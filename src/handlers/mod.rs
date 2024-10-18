use serde::{Deserialize, Serialize};
use sqlx::{types::uuid::Uuid, FromRow};
use time::Date;

pub(crate) mod arcades;
pub(crate) mod cabinets;
mod utils;

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Arcade {
    id: Uuid,
    name: String,
    description: Option<String>,
    create_date: Date,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Game {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Cabinet {
    id: Uuid,
    game_name: String,
    name: String,
    assoc_arcade: Uuid,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Player {
    position: i32,
    name: String,
    assoc_cabinet: Uuid,
}

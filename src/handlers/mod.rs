use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::Date;

pub(crate) mod arcades;
pub(crate) mod cabinets;

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Arcade {
    id: i32,
    name: String,
    description: Option<String>,
    create_date: Date,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Cabinet {
    id: i32,
    game_name: String,
    name: String,
    assoc_arcade: i32,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Player {
    position: i32,
    name: String,
    assoc_cabinet: i32,
}

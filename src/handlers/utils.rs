use sqlx::{query_as, types::Uuid, PgPool};

use super::*;
use crate::error::Error;

pub(super) async fn is_name_in_queue(
    name: &str,
    cabinet_id: Uuid,
    db_pool: &PgPool,
) -> Result<bool, Error> {
    let player: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE name = $1
AND assoc_cabinet = $2
        ",
    )
    .bind(name)
    .bind(cabinet_id)
    .fetch_all(db_pool)
    .await?;

    Ok(!player.is_empty())
}

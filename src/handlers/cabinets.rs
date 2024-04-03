use actix_web::{get, web, HttpResponse, Result};
use serde::Deserialize;
use sqlx::{query_as, PgPool};

use super::*;
use crate::error::Error;

/// `GET /cabinets/*` Routing
pub(crate) fn cabinets_config(cfg: &mut web::ServiceConfig) {
    cfg.service(players);
}

/// List all players in the queue of a cabinet `GET /cabinets/{cabinet_id}/players`
#[get("/{cabinet_id}/players")]
async fn players(
    cabinet_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let players: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE assoc_cabinet = $1
        ",
    )
    .bind(&cabinet_id.into_inner())
    .fetch_all(db_pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(players))
}

use actix_web::{get, web, HttpResponse, Result};
use serde::Deserialize;
use sqlx::{query, query_as, PgPool};

use super::*;
use crate::error::Error;

#[derive(Debug, Deserialize)]
struct NextN {
    n: i32,
}

/// `GET /cabinets/*` Routing
pub(crate) fn cabinets_config(cfg: &mut web::ServiceConfig) {
    cfg.service(players).service(upcoming).service(next);
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

/// List upcoming N players in the queue `GET /cabinets/{cabinet_id}/upcoming?n=N`
#[get("/{cabinet_id}/upcoming")]
async fn upcoming(
    cabinet_id: web::Path<i32>,
    next_n: web::Query<NextN>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let upcoming: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE assoc_cabinet = $1
ORDER BY position
LIMIT $2
        ",
    )
    .bind(cabinet_id.into_inner())
    .bind(next_n.n)
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(upcoming))
}

/// List upcoming N players in the queue, and remove them from the queue
/// `GET /cabinets/{cabinet_id}/next?n=N`
#[get("/{cabinet_id}/next")]
async fn next(
    cabinet_id: web::Path<i32>,
    next_n: web::Query<NextN>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet_id = cabinet_id.into_inner();
    let mut transaction = db_pool.get_ref().begin().await?;

    let next: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE assoc_cabinet = $1
ORDER BY position
LIMIT $2
        ",
    )
    .bind(&cabinet_id)
    .bind(next_n.n)
    .fetch_all(&mut *transaction)
    .await?;

    // Remove the players from the queue
    query(
        "
DELETE FROM arcqueue.players
WHERE assoc_cabinet = $1
AND position < $2
        ",
    )
    .bind(&cabinet_id)
    .bind(next_n.n)
    .fetch_all(&mut *transaction)
    .await?;

    // Reorder the queue
    query(
        "
UPDATE arcqueue.players
SET position = position - $1
WHERE assoc_cabinet = $2
        ",
    )
    .bind(next_n.n)
    .bind(&cabinet_id)
    .fetch_all(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(next))
}

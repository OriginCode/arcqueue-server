use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;
use sqlx::{query, query_as, PgPool};

use super::*;
use crate::error::Error;

#[derive(Debug, Deserialize)]
struct NextN {
    n: i32,
}

#[derive(Debug, Deserialize)]
struct Name {
    name: String,
}

/// `GET /cabinets/*` Routing
pub(crate) fn cabinets_config(cfg: &mut web::ServiceConfig) {
    cfg.service(cabinet)
        .service(players)
        .service(upcoming)
        .service(next)
        .service(join)
        .service(leave);
}

/// Get the cabinet info with `cabinet_id` `GET /cabinets/{cabinet_id}`
#[get("{cabinet_id}")]
async fn cabinet(
    cabinet_id: web::Path<i32>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet: Cabinet = query_as(
        "
SELECT * FROM arcqueue.cabinets
WHERE id = $1
        ",
    )
    .bind(&cabinet_id.into_inner())
    .fetch_one(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(cabinet))
}

/// List all players in the queue of a cabinet `GET /cabinets/{cabinet_id}/players`
#[get("{cabinet_id}/players")]
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
#[get("{cabinet_id}/upcoming")]
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
/// `POST /cabinets/{cabinet_id}/next - n=N`
#[post("{cabinet_id}/next")]
async fn next(
    cabinet_id: web::Path<i32>,
    next_n: web::Form<NextN>,
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
    .execute(&mut *transaction)
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
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(next))
}

/// Join the queue of `cabinet_id` with a name
/// `POST /cabinets/{cabinet_id}/join - name=NAME`
#[post("{cabinet_id}/join")]
async fn join(
    cabinet_id: web::Path<i32>,
    name: web::Form<Name>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet_id = cabinet_id.into_inner();

    // Bails if already in the queue
    let player: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE name = $1
AND assoc_cabinet = $2
        ",
    )
    .bind(&name.name)
    .bind(&cabinet_id)
    .fetch_all(db_pool.get_ref())
    .await?;

    if !player.is_empty() {
        return Ok(HttpResponse::BadRequest().body("Player name exists in the queue"));
    }

    query(
        "
INSERT INTO arcqueue.players
SELECT MAX(position) + 1, $1, $2
FROM arcqueue.players
WHERE assoc_cabinet = $2
        ",
    )
    .bind(&name.name)
    .bind(&cabinet_id)
    .execute(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().body("Done"))
}

/// Leave the queue of `cabinet_id` with a name
/// `POST /cabinets/{cabinet_id}/leave - name=NAME`
#[post("{cabinet_id}/leave")]
async fn leave(
    cabinet_id: web::Path<i32>,
    name: web::Form<Name>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet_id = cabinet_id.into_inner();

    // Bails if not in the queue
    let player: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE name = $1
AND assoc_cabinet = $2
        ",
    )
    .bind(&name.name)
    .bind(&cabinet_id)
    .fetch_all(db_pool.get_ref())
    .await?;

    if player.is_empty() {
        return Ok(HttpResponse::BadRequest().body("Player name not found in the queue"));
    }

    let mut transaction = db_pool.get_ref().begin().await?;

    let (original_position,): (i32,) = query_as(
        "
SELECT position FROM arcqueue.players
WHERE name = $1
AND assoc_cabinet = $2
        ",
    )
    .bind(&name.name)
    .bind(&cabinet_id)
    .fetch_one(&mut *transaction)
    .await?;

    query(
        "
DELETE FROM arcqueue.players
WHERE name = $1
AND assoc_cabinet = $2
        ",
    )
    .bind(&name.name)
    .bind(&cabinet_id)
    .execute(&mut *transaction)
    .await?;

    query(
        "
UPDATE arcqueue.players
SET position = position - 1
WHERE position > $1
AND assoc_cabinet = $2
        ",
    )
    .bind(original_position)
    .bind(&cabinet_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().body("Done"))
}

use actix_web::{delete, get, post, web, HttpResponse};
use serde::Deserialize;
use sqlx::{query, query_as, types::Uuid, PgPool};

use super::*;
use crate::{error::Error, response::Response};
use utils::is_name_in_queue;

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
        .service(info)
        .service(players)
        .service(upcoming)
        .service(next)
        .service(join)
        .service(leave)
        .service(postpone);
}

/// Get the cabinet info with `cabinet_id` `GET /cabinets/{cabinet_id}`
#[get("{cabinet_id}")]
async fn cabinet(
    cabinet_id: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet: Cabinet = query_as(
        "
SELECT * FROM arcqueue.cabinets
WHERE id = $1
        ",
    )
    .bind(Uuid::parse_str(&cabinet_id.into_inner())?)
    .fetch_one(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(cabinet)))
}

/// Get cabinet game information `GET /cabinets/{cabinet_id}/info`
#[get("{cabinet_id}/info")]
async fn info(
    cabinet_id: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let game: Game = query_as(
        "
SELECT g.name AS name, g.description AS description
FROM arcqueue.games g, arcqueue.cabinets c
WHERE g.name = c.game_name
AND c.id = $1
        ",
    )
    .bind(Uuid::parse_str(&cabinet_id.into_inner())?)
    .fetch_one(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(game)))
}

/// List all players in the queue of a cabinet `GET /cabinets/{cabinet_id}/players`
#[get("{cabinet_id}/players")]
async fn players(
    cabinet_id: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let players: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE assoc_cabinet = $1
ORDER BY position
        ",
    )
    .bind(Uuid::parse_str(&cabinet_id.into_inner())?)
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(players)))
}

/// List upcoming N players in the queue `GET /cabinets/{cabinet_id}/upcoming?n=N`
#[get("{cabinet_id}/upcoming")]
async fn upcoming(
    cabinet_id: web::Path<String>,
    next_n: web::Query<NextN>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    // Bails if n is less than 1
    if next_n.n < 1 {
        return Err(Error::NLessThanOne);
    }

    let upcoming: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE assoc_cabinet = $1
ORDER BY position
LIMIT $2
        ",
    )
    .bind(Uuid::parse_str(&cabinet_id.into_inner())?)
    .bind(next_n.n)
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(upcoming)))
}

/// List upcoming N players in the queue, and remove them from the queue
/// `POST /cabinets/{cabinet_id}/next - n=N`
#[post("{cabinet_id}/next")]
async fn next(
    cabinet_id: web::Path<String>,
    next_n: web::Form<NextN>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    // Bails if n is less than 1
    if next_n.n < 1 {
        return Err(Error::NLessThanOne);
    }

    let cabinet_id = Uuid::parse_str(&cabinet_id.into_inner())?;
    let mut transaction = db_pool.get_ref().begin().await?;

    let next: Vec<Player> = query_as(
        "
SELECT * FROM arcqueue.players
WHERE assoc_cabinet = $1
ORDER BY position
LIMIT $2
        ",
    )
    .bind(cabinet_id)
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
    .bind(cabinet_id)
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
    .bind(cabinet_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(Response::success(next)))
}

/// Join the queue of `cabinet_id` with a name
/// `POST /cabinets/{cabinet_id}/join - name=NAME`
#[post("{cabinet_id}/join")]
async fn join(
    cabinet_id: web::Path<String>,
    name: web::Json<Name>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet_id = Uuid::parse_str(&cabinet_id.into_inner())?;

    // Bails if already in the queue
    if is_name_in_queue(&name.name, cabinet_id, db_pool.get_ref()).await? {
        return Err(Error::NameAlreadyInQueue);
    }

    query(
        "
INSERT INTO arcqueue.players
SELECT COALESCE(MAX(position), 0) + 1, $1, $2
FROM arcqueue.players
WHERE assoc_cabinet = $2
        ",
    )
    .bind(&name.name)
    .bind(cabinet_id)
    .execute(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::<()>::default()))
}

/// Leave the queue of `cabinet_id` with a name
/// `DELETE /cabinets/{cabinet_id}/leave - name=NAME`
#[delete("{cabinet_id}/leave")]
async fn leave(
    cabinet_id: web::Path<String>,
    name: web::Json<Name>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet_id = Uuid::parse_str(&cabinet_id.into_inner())?;

    // Bails if not in the queue
    if !is_name_in_queue(&name.name, cabinet_id, db_pool.get_ref()).await? {
        return Err(Error::NameNotInQueue);
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
    .bind(cabinet_id)
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
    .bind(cabinet_id)
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
    .bind(cabinet_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(Response::<()>::default()))
}

/// Swap the position with next one in the queue of `cabinet_id` with a name
/// `POST /cabinets/{cabinet_id}/postpone - name=NAME`
#[post("{cabinet_id}/postpone")]
async fn postpone(
    cabinet_id: web::Path<String>,
    name: web::Json<Name>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinet_id = Uuid::parse_str(&cabinet_id.into_inner())?;

    // Bails if not in the queue
    if !is_name_in_queue(&name.name, cabinet_id, db_pool.get_ref()).await? {
        return Err(Error::NameNotInQueue);
    }

    // Bails if the player is the last one in the queue
    let (is_last,): (bool,) = query_as(
        "
SELECT A.position = MAX(B.position)
FROM arcqueue.players as A, arcqueue.players as B
WHERE A.name = $1
AND A.assoc_cabinet = B.assoc_cabinet
AND A.assoc_cabinet = $2
GROUP BY A.position
        ",
    )
    .bind(&name.name)
    .bind(cabinet_id)
    .fetch_one(db_pool.get_ref())
    .await?;

    if is_last {
        return Err(Error::NameAlreadyLast);
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
    .bind(cabinet_id)
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
    .bind(cabinet_id)
    .execute(&mut *transaction)
    .await?;

    query(
        "
UPDATE arcqueue.players
SET position = position - 1
WHERE position = $1 + 1
AND assoc_cabinet = $2
        ",
    )
    .bind(original_position)
    .bind(cabinet_id)
    .execute(&mut *transaction)
    .await?;

    query(
        "
INSERT INTO arcqueue.players
VALUES ($1, $2, $3)
        ",
    )
    .bind(original_position + 1)
    .bind(&name.name)
    .bind(cabinet_id)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(HttpResponse::Ok().json(Response::<()>::default()))
}

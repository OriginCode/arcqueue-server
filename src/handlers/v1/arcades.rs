use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::{query_as, types::Uuid, PgPool};

use super::*;
use crate::{error::Error, response::Response};

#[derive(Debug, Deserialize)]
struct Name {
    name: String,
}

/// `GET /arcades/*` Routing
pub(crate) fn arcades_config(cfg: &mut web::ServiceConfig) {
    cfg.service(all)
        .service(search)
        .service(arcade)
        .service(cabinets);
}

/// Search for all arcades `GET /arcades`
#[get("")]
async fn all(db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let arcades: Vec<Arcade> = query_as("SELECT * FROM arcqueue.arcades WHERE is_public = true")
        .fetch_all(db_pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(Response::success(arcades)))
}

/// Search for arcades by name `GET /arcades/search?name=NAME`
#[get("search")]
async fn search(name: web::Query<Name>, db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let arcades: Vec<Arcade> = query_as(
        "
SELECT * FROM arcqueue.arcades
WHERE SIMILARITY(name, $1) > 0.4
AND is_public = true
        ",
    )
    .bind(&name.name)
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(arcades)))
}

/// Get arcade info with `arcade_id` `GET /arcades/{arcade_id}`
#[get("{arcade_id}")]
async fn arcade(
    arcade_id: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let arcade: Arcade = query_as(
        "
SELECT * FROM arcqueue.arcades
WHERE id = $1
        ",
    )
    .bind(Uuid::parse_str(&arcade_id.into_inner())?)
    .fetch_one(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(arcade)))
}

/// List all cabinets in an arcade `GET /arcades/{arcade_id}/cabinets`
#[get("{arcade_id}/cabinets")]
async fn cabinets(
    arcade_id: web::Path<String>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let cabinets: Vec<Cabinet> = query_as(
        "
SELECT * FROM arcqueue.cabinets
WHERE assoc_arcade = $1
        ",
    )
    .bind(Uuid::parse_str(&arcade_id.into_inner())?)
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(Response::success(cabinets)))
}

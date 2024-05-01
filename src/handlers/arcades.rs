use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::{query_as, PgPool, types::Uuid};

use super::*;
use crate::error::Error;

#[derive(Debug, Deserialize)]
struct Name {
    name: String,
}

/// `GET /arcades/*` Routing
pub(crate) fn arcades_config(cfg: &mut web::ServiceConfig) {
    cfg.service(all)
        .service(arcade)
        .service(search)
        .service(cabinets);
}

/// Search for all arcades `GET /arcades`
#[get("")]
async fn all(db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let arcades: Vec<Arcade> = query_as("SELECT * FROM arcqueue.arcades")
        .fetch_all(db_pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(arcades))
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

    Ok(HttpResponse::Ok().json(arcade))
}

/// Search for arcades by name `GET /arcades/search?name=NAME`
#[get("search")]
async fn search(name: web::Query<Name>, db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let arcades: Vec<Arcade> = query_as(
        "
SELECT * FROM arcqueue.arcades
WHERE SIMILARITY(name, $1) > 0.4
        ",
    )
    .bind(&name.name)
    .fetch_all(db_pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(arcades))
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

    Ok(HttpResponse::Ok().json(cabinets))
}

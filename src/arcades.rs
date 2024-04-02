use crate::error::Error;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, PgPool};
use time::Date;

#[derive(Debug, Deserialize, Serialize, FromRow)]
struct Arcade {
    id: i32,
    name: String,
    description: Option<String>,
    create_date: Date,
}

#[derive(Debug, Deserialize)]
struct Name {
    name: String,
}

pub(crate) fn arcades_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/all").route(web::get().to(all)))
        .service(web::resource("/search").route(web::get().to(search)));
}

async fn all(db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> {
    let arcades: Vec<Arcade> = query_as("SELECT * FROM arcqueue.arcades")
        .fetch_all(db_pool.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(arcades))
}

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

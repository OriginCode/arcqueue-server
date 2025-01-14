use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod cli;
mod error;
mod ping;

// Routing
mod handlers;

use handlers::*;

#[actix_web::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    let args = cli::Args::parse();

    // Set up database connection
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&args.pg_url)
        .await?;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(ping::ping)
            .service(web::scope("/arcades").configure(arcades::arcades_config))
            .service(web::scope("/cabinets").configure(cabinets::cabinets_config))
    })
    .bind((
        args.host.unwrap_or("localhost".to_owned()),
        args.port.unwrap_or(8701),
    ))?
    .run()
    .await?;

    Ok(())
}

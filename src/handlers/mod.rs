use actix_web::web;

mod v1;
use v1::*;

/// `GET /arcades/*` Routing
pub(crate) fn apiv1_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/arcades").configure(arcades::arcades_config))
        .service(web::scope("/cabinets").configure(cabinets::cabinets_config));
}

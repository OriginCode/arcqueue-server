use actix_web::{HttpResponse, Responder};

#[actix_web::get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

use actix_web::{get, HttpResponse, Responder};

#[get("")]
async fn recipes() -> impl Responder {
    return HttpResponse::Ok().body("recipes!");
}

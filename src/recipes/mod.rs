use actix_web::web;

mod routes;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(routes::recipes);
}

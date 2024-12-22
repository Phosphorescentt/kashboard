use actix_web::web;

pub mod council;
pub mod octopus;
pub mod routes;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(routes::bins);
}

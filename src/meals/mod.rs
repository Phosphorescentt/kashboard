use actix_web::web;

mod model;
mod routes;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(routes::recipes_list);
    cfg.service(routes::recipe_get);
    cfg.service(routes::plan);
    cfg.service(routes::persist_plan);
}

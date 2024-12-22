use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use lazy_static::lazy_static;
use sqlx::sqlite::SqlitePool;
use std::env;
use tera::Tera;

pub mod calendar;
pub mod recipes;
pub mod utilities;

pub struct AppState {
    pool: SqlitePool,
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = SqlitePool::connect(env::var("DATABASE_URL").unwrap().as_str())
        .await
        .unwrap();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { pool: pool.clone() }))
            .service(web::scope("/calendar").configure(calendar::init))
            .service(web::scope("/utilities").configure(utilities::init))
            .service(web::scope("/recipes").configure(recipes::init))
            .wrap(Logger::default())
    })
    .bind(("192.168.0.86", 8080))?
    .bind(("127.0.0.1", 8080))?
    .run();

    server.await
}

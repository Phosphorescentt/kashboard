use crate::AppState;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx;

#[get("")]
async fn index() -> impl Responder {
    return HttpResponse::Ok().body("calendar");
}

#[get("/test")]
async fn calendar(state: web::Data<AppState>) -> impl Responder {
    let p = state.pool.clone();

    let data = sqlx::query(r#"SELECT * FROM recipes"#)
        .execute(&p)
        .await
        .map_err(|err: sqlx::Error| err.to_string());

    return HttpResponse::Ok().body(format!("hi baby !!! {:?}", data));
}

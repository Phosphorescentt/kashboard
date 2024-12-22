use actix_web::{get, web, HttpResponse, Responder};
use tera::Context;

use crate::{AppState, TEMPLATES};

use super::model;

#[get("")]
async fn recipes(state: web::Data<AppState>) -> impl Responder {
    let p = state.pool.clone();

    let q = sqlx::query_as::<_, model::Recipe>(r#"SELECT * FROM recipes;"#);
    let recipes: Vec<model::Recipe> = q.fetch_all(&p).await.unwrap();

    let mut context = Context::new();
    context.insert("recipes", &recipes);
    let content = TEMPLATES.render("recipes/list.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

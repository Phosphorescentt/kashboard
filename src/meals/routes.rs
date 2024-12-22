use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use tera::Context;

use crate::{AppState, TEMPLATES};

use super::model;

#[get("")]
async fn recipes_list(state: web::Data<AppState>) -> impl Responder {
    let p = state.pool.clone();

    let recipes = sqlx::query_as::<_, model::Recipe>(r#"SELECT * FROM recipes;"#)
        .fetch_all(&p)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("recipes", &recipes);
    let content = TEMPLATES.render("meals/list.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

#[get("/recipe/{id}")]
async fn recipe_get(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let id: u32 = req.match_info().get("id").unwrap().parse().unwrap();
    let p = state.pool.clone();

    let recipe = sqlx::query_as!(model::Recipe, r#"SELECT * FROM recipes WHERE id = ?"#, id)
        .fetch_one(&p)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("recipe", &recipe);

    let content = TEMPLATES.render("meals/recipe.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

#[get("/plan")]
async fn plan(state: web::Data<AppState>) -> impl Responder {
    let p = state.pool.clone();

    let recipes = sqlx::query_as::<_, model::Recipe>(r#"SELECT * FROM recipes;"#)
        .fetch_all(&p)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("recipes", &recipes);
    let content = TEMPLATES.render("meals/plan.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

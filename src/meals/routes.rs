use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use log::info;
use serde::Deserialize;
use tera::Context;

use crate::{AppState, TEMPLATES};

use super::model;

#[derive(Debug, Deserialize)]
struct ShoppingListRequest {
    recipe_ids: String,
}

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

    let ingredients: Vec<model::IngredientWithCountUnit> = sqlx::query_as!(
        model::IngredientWithCountUnit,
        r#"
            SELECT i.id as id,
                i.name as name,
                Sum(ria.count) as count,
                ria.unit as unit
            FROM ingredients AS i
                JOIN recipe_ingredients_associations AS ria
                    ON i.id = ria.ingredient_id
            WHERE ria.recipe_id = ?
            GROUP BY ria.ingredient_id,
                    ria.unit
            "#,
        id,
    )
    .fetch_all(&p)
    .await
    .unwrap();

    let mut context = Context::new();
    context.insert("recipe", &recipe);
    context.insert("ingredients", &ingredients);

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

fn validate_ids_string(s: String) -> Option<Vec<i64>> {
    let parts = s.split(",");
    let parsed_parts: Vec<Option<i64>> = parts
        .map(|subs| match subs.parse::<i64>() {
            Ok(v) => Some(v),
            Err(_) => None,
        })
        .collect();

    // If everything parsed into an i64, it's valid.
    if parsed_parts.iter().all(|e| {
        if let Some(_) = e {
            return true;
        } else {
            return false;
        }
    }) {
        return Some(parsed_parts.iter().map(|e| e.unwrap()).collect());
    } else {
        return None;
    }
}

#[post("/create-shopping-list")]
async fn persist_plan(
    form: web::Form<ShoppingListRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Log the request body here so we can see if we're getting data!
    let p = state.pool.clone();

    let is_valid = validate_ids_string(form.recipe_ids.clone());
    // This is almost definitely
    if let Some(ids) = is_valid {
        let ingredients = sqlx::query_as::<_, model::IngredientWithCountUnit>(
            format!(
                r#"
            SELECT i.id as id,
                i.name as name,
                Sum(ria.count) as count,
                ria.unit as unit
            FROM ingredients AS i
                JOIN recipe_ingredients_associations AS ria
                    ON i.id = ria.ingredient_id
            WHERE ria.recipe_id in ({}) 
            GROUP BY ria.ingredient_id,
                    ria.unit
            "#,
                ids.iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            )
            .as_str(),
        )
        .fetch_all(&p)
        .await
        .unwrap();

        let mut context = Context::new();
        context.insert("ingredients", &ingredients);

        let content = TEMPLATES
            .render("meals/shopping_list.html", &context)
            .unwrap();
        return HttpResponse::Ok().body(content);
    } else {
        info!("Invalid recipe_ids string provided. Returning 400.");
        return HttpResponse::BadRequest().into();
    }
}

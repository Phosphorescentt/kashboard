use actix_web::{
    get, post, route,
    web::{self, Redirect},
    HttpRequest, HttpResponse, Responder,
};
use log::{debug, info};
use serde::Deserialize;
use std::collections::HashMap;
use tera::Context;

use crate::{AppState, TEMPLATES};

use super::model;

#[derive(Debug, Deserialize)]
struct ShoppingListRequest {
    recipe_ids: String,
}

#[derive(Debug, Deserialize)]
struct CreateRecipeFormData {
    recipe_name: String,
    recipe_time_minutes: String,
    recipe_instructions: String,
    ingredients: String,
}

#[derive(Debug, Deserialize)]
struct IngredientsCounts {
    name: String,
    count: i64,
    unit: String,
}

// Views

#[route("", method = "GET", method = "POST")]
async fn recipes_list(state: web::Data<AppState>) -> impl Responder {
    let p = state.pool.clone();

    let recipes = sqlx::query_as::<_, model::Recipe>(r#"SELECT * FROM recipes;"#)
        .fetch_all(&p)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("recipes", &recipes);
    let content = TEMPLATES.render("recipes/list.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

#[route("/recipe/{id}", method = "GET", method = "POST")]
async fn recipe_get(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let id: u32 = req.match_info().get("id").unwrap().parse().unwrap();
    let p = state.pool.clone();

    let recipe = sqlx::query_as!(model::Recipe, r#"SELECT * FROM recipes WHERE id = ?"#, id)
        .fetch_one(&p)
        .await
        .unwrap();

    let ingredients = sqlx::query_as!(
        model::IngredientWithCountUnit,
        r#"
            SELECT i.id as id,
                i.name as name,
                Sum(ria.count) as "count: _",
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

    let content = TEMPLATES.render("recipes/recipe.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

#[get("/create")]
async fn create() -> impl Responder {
    let context = Context::new();
    let content = TEMPLATES
        .render("recipes/recipe_create.html", &context)
        .unwrap();
    return HttpResponse::Ok().body(content);
}

#[post("/create")]
async fn create_db_records(
    form: web::Json<CreateRecipeFormData>,
    state: web::Data<AppState>,
) -> impl Responder {
    let p = state.pool.clone();
    // parse ingredients
    let ingredients = serde_json::from_str::<Vec<IngredientsCounts>>(&form.ingredients).unwrap();

    // create recipe
    let recipe_id = sqlx::query!(
        r#"INSERT INTO recipes (name, time_minutes, instructions) VALUES (?, ?, ?) RETURNING id"#,
        form.recipe_name,
        form.recipe_time_minutes,
        form.recipe_instructions
    )
    .fetch_one(&p)
    .await
    .unwrap()
    .id;

    // upsert ingredients and create associations
    for ingredient in ingredients {
        let id_row_result = sqlx::query!(
            r#"SELECT id FROM ingredients WHERE name LIKE ?"#,
            ingredient.name,
        )
        .fetch_one(&p)
        .await;

        let ingredient_id: i64;
        match id_row_result {
            Ok(id_row) => {
                // If there's already something in the DB, use that ID
                ingredient_id = id_row.id;
            }
            Err(_) => {
                // Otherwise create a new one.
                let new_record = sqlx::query!(
                    r#"INSERT INTO ingredients (name) VALUES (?) RETURNING id"#,
                    ingredient.name
                )
                .fetch_one(&p)
                .await
                .unwrap();

                ingredient_id = new_record.id;
            }
        }

        sqlx::query!(
            r#"INSERT INTO recipe_ingredients_associations (recipe_id, ingredient_id, count, unit) VALUES (?, ?, ?, ?)"#,
            recipe_id,
            ingredient_id,
            ingredient.count,
            ingredient.unit,
        ).fetch_all(&p).await.unwrap();
    }

    let response = HttpResponse::Ok()
        .insert_header(("HX-Redirect", format!("/recipes/recipe/{}", recipe_id)))
        .body("None");

    return response;
    // return Redirect::to(format!("/recipes/recipe/{}", recipe_id));
}

#[post("/recipe/delete/{id}")]
async fn recipe_delete(req: HttpRequest, state: web::Data<AppState>) -> impl Responder {
    let id: u32 = req.match_info().get("id").unwrap().parse().unwrap();
    let p = state.pool.clone();

    let _deleted = sqlx::query!(r#"DELETE FROM recipes WHERE id = ?"#, id)
        .fetch_all(&p)
        .await
        .unwrap();

    // TODO: Fix this redirect. For some reason the HTML renders raw after the redirect?
    return Redirect::to("/recipes");
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

    let content = TEMPLATES.render("recipes/plan.html", &context).unwrap();
    return HttpResponse::Ok().body(content);
}

// COMPONENTS

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
async fn create_shopping_list(
    form: web::Form<ShoppingListRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    // Log the request body here so we can see if we're getting data!
    let p = state.pool.clone();

    let is_valid = validate_ids_string(form.recipe_ids.clone());
    // This is almost definitely
    if let Some(ids) = is_valid {
        info!("recipe_ids: {}", form.recipe_ids.clone());
        let mut ingredients_store: HashMap<(i64, String, String), i64> = HashMap::new();

        for id in ids.iter() {
            // do stuff
            let ingredients = sqlx::query_as!(
                model::IngredientWithCountUnit,
                r#"SELECT 
                    i.id AS id,
                    i.name AS name,
                    ria.count as count,
                    ria.unit as unit
                FROM ingredients AS i
                JOIN recipe_ingredients_associations AS ria
                    ON i.id = ria.ingredient_id
                WHERE ria.recipe_id = ?"#,
                id,
            )
            .fetch_all(&p)
            .await
            .unwrap();

            for ingredient in ingredients.iter() {
                if let Some(current_count) = ingredients_store.get(&(
                    ingredient.id,
                    ingredient.name.clone(),
                    ingredient.unit.clone(),
                )) {
                    ingredients_store.insert(
                        (
                            ingredient.id,
                            ingredient.name.clone(),
                            ingredient.unit.clone(),
                        ),
                        current_count + ingredient.count.unwrap(),
                    );
                } else {
                    ingredients_store.insert(
                        (
                            ingredient.id,
                            ingredient.name.clone(),
                            ingredient.unit.clone(),
                        ),
                        ingredient.count.unwrap(),
                    );
                }
            }
        }

        debug!("{:?}", ingredients_store);

        for k in ingredients_store.keys() {
            let v = ingredients_store.get(k).unwrap();
        }
        let ingredients = ingredients_store
            .keys()
            .map(|k| {
                let v = ingredients_store.get(k).unwrap();
                return model::IngredientWithCountUnit {
                    id: k.0,
                    name: k.1.clone(),
                    count: Some(*v),
                    unit: k.2.clone(),
                };
            })
            .collect::<Vec<model::IngredientWithCountUnit>>();

        let mut context = Context::new();
        context.insert("ingredients", &ingredients);

        let content = TEMPLATES
            .render("recipes/shopping_list.html", &context)
            .unwrap();
        return HttpResponse::Ok().body(content);
    } else {
        info!("Invalid recipe_ids string provided. Returning 400.");
        return HttpResponse::BadRequest().into();
    }
}

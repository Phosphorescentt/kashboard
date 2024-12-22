use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub time_minutes: Option<i64>,
    pub instructions: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Ingredients {
    id: u32,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct RecipeIngredientsAssociations {
    id: u32,
    recipe_id: u32,
    ingredient_id: u32,
    count: u32,
}

struct RecipeDates {
    id: u32,
    recipe_id: u32,
    date: chrono::DateTime<chrono::Utc>,
}

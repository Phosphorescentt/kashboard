CREATE TABLE recipes (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    time_minutes INTEGER,
    instructions TEXT
);

CREATE TABLE ingredients (
    id INTEGER PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE recipe_ingredients_associations (
    id INTEGER PRIMARY KEY NOT NULL,
    recipe_id INTEGER NOT NULL,
    ingredient_id INTEGER NOT NULL,
    count INTEGER NOT NULL,
    unit VARCHAR(255) NOT NULL,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id),
    FOREIGN KEY (ingredient_id) REFERENCES ingredients(id),
    UNIQUE (recipe_id, ingredient_id, unit)
);

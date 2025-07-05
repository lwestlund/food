PRAGMA foreign_keys = ON;

CREATE TABLE source(
       id INTEGER PRIMARY KEY NOT NULL,
       name TEXT NOT NULL,
       url TEXT
);

CREATE TABLE meal_type(
       id INTEGER PRIMARY KEY NOT NULL,
       type_name TEXT UNIQUE NOT NULL
);

CREATE TABLE recipe(
       id INTEGER PRIMARY KEY NOT NULL,
       title TEXT UNIQUE NOT NULL,
       description TEXT NOT NULL,
       meal_type_id INTEGER NOT NULL,
       source_id INTEGER NOT NULL,
       creation_date TEXT NOT NULL DEFAULT (STRFTIME('%Y-%m-%d', 'NOW')),
       FOREIGN KEY (meal_type_id) REFERENCES meal_type(id),
       FOREIGN KEY (source_id) REFERENCES source(id)
);

CREATE TABLE instruction (
       id INTEGER PRIMARY KEY NOT NULL,
       step_number INTEGER NOT NULL,
       description TEXT NOT NULL,
       recipe_id INTEGER NOT NULL,
       FOREIGN KEY (recipe_id) REFERENCES recipe(id)
);

CREATE TABLE ingredient(
       id INTEGER PRIMARY KEY NOT NULL,
       name TEXT UNIQUE NOT NULL
);
CREATE TABLE measurement(
       id INTEGER PRIMARY KEY NOT NULL,
       unit TEXT UNIQUE NOT NULL
);
CREATE TABLE recipe_ingredient(
       id INTEGER PRIMARY KEY NOT NULL,
       quantity REAL NOT NULL,
       recipe_id INTEGER NOT NULL,
       ingredient_id INTEGER NOT NULL,
       measurement_id INTEGER NOT NULL,
       FOREIGN KEY (recipe_id) REFERENCES recipe(id),
       FOREIGN KEY (ingredient_id) REFERENCES ingredient(id),
       FOREIGN KEY (measurement_id) REFERENCES measurement(id)
);

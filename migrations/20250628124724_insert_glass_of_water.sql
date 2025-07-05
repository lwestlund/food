INSERT INTO source
(name)
VALUES
('Cool source');

INSERT INTO meal_type
(type_name)
VALUES
('Drink');

INSERT INTO ingredient
(name)
VALUES
('water'),
('any drinking glass');

INSERT INTO measurement
(unit)
VALUES
('dl'),
('piece');

INSERT INTO recipe
(title, description, meal_type_id, source_id, creation_date)
VALUES
('Glass of water', 'Refreshing, isn''t it?', 1, 1, '2025-01-19');

INSERT INTO recipe_ingredient
(quantity, recipe_id, ingredient_id, measurement_id)
VALUES
(25, 1, 1, 1),
(1, 1, 2, 2);

INSERT INTO instruction
(step_number, description, recipe_id)
VALUES
(1, 'Pour the water into the glass', 1),
(2, 'Enjoy the nice water', 1);

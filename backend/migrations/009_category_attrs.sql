-- Category-specific attribute definitions for AI organize
-- Add food_type (nutrition), electronics_type (electronics),
-- body_parts_secondary (global), and update body_parts options to CDDA style

INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, category_scope, sort_order)
VALUES ('food_type', '食品类型', 'text', '{"options":["能量胶","能量棒","巧克力","果泥","威化","饼干","坚果","肉干","饮品","补剂","其他"]}', '3', 30);

INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, category_scope, sort_order)
VALUES ('electronics_type', '电子类型', 'text', '{"options":["照明","通讯","导航","摄影","电源","穿戴","其他"]}', '4', 31);

INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order)
VALUES ('body_parts_secondary', '副覆盖', 'text', '{"options":["头","眼","口","颈","躯干","手臂","手","腿","脚","腰","臀","全身"]}', 12);

-- Update body_parts options to CDDA style
UPDATE attribute_definitions
SET config = '{"options":["头","眼","口","颈","躯干","手臂","手","腿","脚","腰","臀","全身"]}'
WHERE key = 'body_parts';

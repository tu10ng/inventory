-- Fix category_scope for attribute definitions
-- Most attributes were created with empty scope (global), but they only make sense for specific categories

-- 服装专属 (category 1)
UPDATE attribute_definitions SET category_scope = '1' WHERE key = 'warmth_rating';
UPDATE attribute_definitions SET category_scope = '1' WHERE key = 'breathable';
UPDATE attribute_definitions SET category_scope = '1' WHERE key = 'season';

-- 服装 + 装备 (categories 1, 2)
UPDATE attribute_definitions SET category_scope = '1,2' WHERE key = 'encumbrance';
UPDATE attribute_definitions SET category_scope = '1,2' WHERE key = 'env_protection';
UPDATE attribute_definitions SET category_scope = '1,2' WHERE key = 'durability';
UPDATE attribute_definitions SET category_scope = '1,2' WHERE key = 'material';
UPDATE attribute_definitions SET category_scope = '1,2' WHERE key = 'body_parts';
UPDATE attribute_definitions SET category_scope = '1,2' WHERE key = 'body_parts_secondary';

-- 服装 + 装备 + 电子 (categories 1, 2, 4)
UPDATE attribute_definitions SET category_scope = '1,2,4' WHERE key = 'waterproof';

-- 电子专属 (category 4) — fixes '[]' bug from AI-created attributes
UPDATE attribute_definitions SET category_scope = '4' WHERE key = 'storage';
UPDATE attribute_definitions SET category_scope = '4' WHERE key = 'ram';

-- 命名去歧义: storage_ml → 液体容量 (区别于 storage 存储容量)
UPDATE attribute_definitions SET label = '液体容量' WHERE key = 'storage_ml';

-- Seed display rules (idempotent)
INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (1, '服装按部位', 1, 'body_parts', 'name', 'asc', '["tag","name","brand","body_parts","waterproof","season","warmth_rating"]', 0, '{"mode":"list","summary_fields":[]}');

INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (2, '按重量排序', NULL, '', 'weight_grams', 'asc', '["tag","name","brand","weight_grams","material"]', 1, '{"mode":"list","summary_fields":[]}');

INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (3, '核心视图', NULL, '', 'name', 'asc', '["tag","name","brand"]', 2, '{"mode":"list","summary_fields":[]}');

-- Summary mode: 尺码汇总 (group by brand, show size/color summary)
INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (4, '尺码汇总', 1, 'brand', 'name', 'asc', '["tag","brand","size","color"]', 10, '{"mode":"summary","summary_fields":["size","color"]}');

-- Update existing rules to have proper config (for DBs that already had the seed without config)
UPDATE display_rules SET config = '{"mode":"list","summary_fields":[]}' WHERE config = '{}';

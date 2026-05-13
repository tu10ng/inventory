CREATE TABLE IF NOT EXISTS display_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    group_by_key TEXT NOT NULL DEFAULT '',
    sort_by_key TEXT NOT NULL DEFAULT '',
    sort_dir TEXT NOT NULL DEFAULT 'asc',
    visible_columns TEXT NOT NULL DEFAULT '[]',
    sort_order INTEGER NOT NULL DEFAULT 0,
    config TEXT NOT NULL DEFAULT '{}'
);

-- Seed: 实用展示规则

-- 服装按覆盖部位分组
INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (1, '服装按部位', 1, 'body_parts', 'name', 'asc', '["tag","name","brand","body_parts","waterproof","season","warmth_rating"]', 0, '{"mode":"list","summary_fields":[]}');

-- 不筛选分类，按重量排序
INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (2, '按重量排序', NULL, '', 'weight_grams', 'asc', '["tag","name","brand","weight_grams","material"]', 1, '{"mode":"list","summary_fields":[]}');

-- 不筛选分类，不分组，只显示核心列
INSERT OR IGNORE INTO display_rules (id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config)
VALUES (3, '核心视图', NULL, '', 'name', 'asc', '["tag","name","brand"]', 2, '{"mode":"list","summary_fields":[]}');

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    brand TEXT NOT NULL DEFAULT '',
    model TEXT NOT NULL DEFAULT '',
    category_id INTEGER NOT NULL REFERENCES categories(id),
    default_qty INTEGER NOT NULL DEFAULT 1,
    notes TEXT NOT NULL DEFAULT '',
    tag_id INTEGER REFERENCES tags(id)
);

CREATE TABLE IF NOT EXISTS activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS activity_slots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    slot_name TEXT NOT NULL,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    is_essential INTEGER NOT NULL DEFAULT 1,
    default_qty INTEGER NOT NULL DEFAULT 1,
    notes TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS activity_slot_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slot_id INTEGER NOT NULL REFERENCES activity_slots(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    UNIQUE(slot_id, tag_id)
);

CREATE TABLE IF NOT EXISTS tips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS people (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    activity_id INTEGER REFERENCES activities(id),
    start_date TEXT NOT NULL DEFAULT '',
    end_date TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'planning'
);

CREATE TABLE IF NOT EXISTS trip_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    item_id INTEGER REFERENCES items(id) ON DELETE SET NULL,
    custom_name TEXT NOT NULL DEFAULT '',
    person_id INTEGER REFERENCES people(id) ON DELETE SET NULL,
    qty INTEGER NOT NULL DEFAULT 1,
    checked INTEGER NOT NULL DEFAULT 0,
    item_status TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_essential INTEGER NOT NULL DEFAULT 0,
    slot_id INTEGER REFERENCES activity_slots(id)
);

-- Incremental migration: add new columns (safe to re-run via ALTER TABLE tolerance in db.rs)
ALTER TABLE items ADD COLUMN tag_id INTEGER REFERENCES tags(id);
ALTER TABLE trip_items ADD COLUMN is_essential INTEGER NOT NULL DEFAULT 0;
ALTER TABLE trip_items ADD COLUMN slot_id INTEGER REFERENCES activity_slots(id);

-- Item physical attributes (CDDA-style)
ALTER TABLE items ADD COLUMN warmth_rating INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN material TEXT NOT NULL DEFAULT '';
ALTER TABLE items ADD COLUMN encumbrance INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN waterproof INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN weight_grams INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN season TEXT NOT NULL DEFAULT '';
ALTER TABLE items ADD COLUMN body_parts TEXT NOT NULL DEFAULT '';
ALTER TABLE items ADD COLUMN env_protection INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN durability INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN storage_ml INTEGER NOT NULL DEFAULT 0;
ALTER TABLE items ADD COLUMN breathable INTEGER NOT NULL DEFAULT 0;

-- Seed categories (only inserted on fresh DB, user can manage via CRUD after)
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (1, '服装', '👕', 1);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (2, '装备', '🎒', 2);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (3, '营养', '🍫', 3);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (4, '电子', '🔋', 4);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (5, '急救', '🩹', 5);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (6, '洗漱', '🧴', 6);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (7, '证件', '📄', 7);
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES (8, '其他', '📦', 8);

-- Status definitions (dynamic status values)
CREATE TABLE IF NOT EXISTS status_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope TEXT NOT NULL,
    value TEXT NOT NULL,
    label TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '',
    icon TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(scope, value)
);

-- Seed item statuses
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('item', '', '无', 0);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('item', 'need_buy', '需购买', 1);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('item', 'need_find', '需寻找', 2);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('item', 'need_charge', '需充电', 3);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('item', 'need_fetch', '需取回', 4);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('item', 'need_give', '需带给', 5);

-- Seed trip statuses
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('trip', 'planning', '计划中', 1);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('trip', 'packing', '打包中', 2);
INSERT OR IGNORE INTO status_definitions (scope, value, label, sort_order) VALUES ('trip', 'done', '已完成', 3);

-- Attribute definitions (dynamic item properties)
CREATE TABLE IF NOT EXISTS attribute_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    attr_type TEXT NOT NULL DEFAULT 'number',
    config TEXT NOT NULL DEFAULT '{}',
    category_scope TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

-- Seed attribute definitions (migrated from hardcoded columns)
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('warmth_rating', '保暖', 'bar', '{"max":50}', 1);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('weight_grams', '重量', 'weight', '{"suffix":"g"}', 2);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('encumbrance', '累赘', 'bar', '{"max":10}', 3);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('waterproof', '防水', 'bool', '{}', 4);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('breathable', '透气', 'bool', '{}', 5);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('env_protection', '环境防护', 'stars', '{"max":5}', 6);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('durability', '耐久', 'stars', '{"max":5}', 7);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('storage_ml', '容量', 'number', '{"suffix":"ml"}', 8);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('material', '材质', 'text', '{}', 9);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('season', '适用季节', 'text', '{"options":["春","夏","秋","冬"]}', 10);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('body_parts', '覆盖部位', 'text', '{"options":["头","躯干","腿","脚","手"]}', 11);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('default_qty', '数量', 'number', '{}', 12);

-- Add attrs JSON column to items
ALTER TABLE items ADD COLUMN attrs TEXT NOT NULL DEFAULT '{}';

ALTER TABLE activity_slots ADD COLUMN default_item_id INTEGER REFERENCES items(id) ON DELETE SET NULL

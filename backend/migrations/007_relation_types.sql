-- Dynamic relation system: relation_types + item_relations
-- A new meta-layer alongside attribute_definitions and status_definitions.

CREATE TABLE IF NOT EXISTS relation_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL DEFAULT '',
    color TEXT NOT NULL DEFAULT '',
    icon TEXT NOT NULL DEFAULT '',
    bidirectional INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS item_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    target_item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    relation_type_id INTEGER NOT NULL REFERENCES relation_types(id) ON DELETE CASCADE,
    notes TEXT NOT NULL DEFAULT '',
    UNIQUE(source_item_id, target_item_id, relation_type_id)
);

-- Seed relation types
INSERT OR IGNORE INTO relation_types (id, name, label, color, icon, bidirectional, sort_order)
VALUES (1, 'depends_on', '依赖', '#e94560', '🔗', 0, 1);

INSERT OR IGNORE INTO relation_types (id, name, label, color, icon, bidirectional, sort_order)
VALUES (2, 'goes_with', '搭配', '#533483', '👔', 1, 2);

INSERT OR IGNORE INTO relation_types (id, name, label, color, icon, bidirectional, sort_order)
VALUES (3, 'alternative_to', '替代', '#0f3460', '🔄', 1, 3);

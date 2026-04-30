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

CREATE TABLE IF NOT EXISTS activity_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    is_essential INTEGER NOT NULL DEFAULT 1,
    default_qty INTEGER NOT NULL DEFAULT 1,
    notes TEXT NOT NULL DEFAULT ''
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
    status TEXT NOT NULL DEFAULT 'planning' CHECK(status IN ('planning', 'packing', 'done'))
);

CREATE TABLE IF NOT EXISTS trip_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    item_id INTEGER REFERENCES items(id),
    custom_name TEXT NOT NULL DEFAULT '',
    person_id INTEGER REFERENCES people(id),
    qty INTEGER NOT NULL DEFAULT 1,
    checked INTEGER NOT NULL DEFAULT 0,
    item_status TEXT NOT NULL DEFAULT '' CHECK(item_status IN ('', 'need_buy', 'need_find', 'need_charge', 'need_fetch', 'need_give')),
    notes TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_essential INTEGER NOT NULL DEFAULT 0,
    slot_id INTEGER REFERENCES activity_slots(id)
);

-- Seed categories
INSERT OR IGNORE INTO categories (id, name, icon, sort_order) VALUES
    (1, '服装', '👕', 1),
    (2, '装备', '🎒', 2),
    (3, '营养', '🍫', 3),
    (4, '电子', '🔋', 4),
    (5, '急救', '🩹', 5),
    (6, '洗漱', '🧴', 6),
    (7, '证件', '📄', 7),
    (8, '其他', '📦', 8);

-- Seed tags (21 unique tags, items 15/16 share tag 15, items 20/21 share tag 19)
INSERT OR IGNORE INTO tags (id, name, category_id, sort_order) VALUES
    (1,  '软壳衣',     1, 1),
    (2,  '软壳裤',     1, 2),
    (3,  '硬壳/雨衣',  1, 3),
    (4,  '速干衣',     1, 4),
    (5,  '抓绒',       1, 5),
    (6,  '羽绒',       1, 6),
    (7,  '袜子',       1, 7),
    (8,  '徒步鞋',     2, 1),
    (9,  '登山杖',     2, 2),
    (10, '轻装包',     2, 3),
    (11, '头灯',       2, 4),
    (12, '墨镜',       2, 5),
    (13, '防晒霜',     6, 1),
    (14, '充电宝',     4, 1),
    (15, '充电线',     4, 2),
    (16, '创口贴',     5, 1),
    (17, '碘伏棉签',   5, 2),
    (18, '零食',       3, 1),
    (19, '能量胶',     3, 2),
    (20, '电解质',     3, 3),
    (21, '高反药',     3, 4);

-- Seed items (with tag_id)
INSERT OR IGNORE INTO items (id, name, brand, model, category_id, default_qty, tag_id) VALUES
    (1,  '软壳衣',       'SIMOND',      '',           1, 1, 1),
    (2,  '软壳裤',       '',            'mh500',      1, 1, 2),
    (3,  '硬壳衣',       '三峰',        '15D雨衣',    1, 1, 3),
    (4,  '速干衣',       '',            '羊毛',       1, 2, 4),
    (5,  '抓绒',         '',            'mh100摇粒绒', 1, 1, 5),
    (6,  '排骨羽绒',     'forclaz',     'trek100',    1, 1, 6),
    (7,  '袜子',         '',            '羊毛',       1, 2, 7),
    (8,  '徒步鞋',       '',            '',           2, 2, 8),
    (9,  '登山杖',       'naturehike',  '初雪ext1',   2, 2, 9),
    (10, '轻装包',       '',            '',           2, 1, 10),
    (11, '头灯',         '',            '',           2, 1, 11),
    (12, '墨镜和眼镜布', '',            '',           2, 1, 12),
    (13, '防晒霜',       '',            '',           6, 1, 13),
    (14, '充电宝',       '',            '',           4, 1, 14),
    (15, '手表充电线',   '',            '',           4, 1, 15),
    (16, '手机充电线',   '',            '',           4, 1, 15),
    (17, '创口贴',       '',            '',           5, 1, 16),
    (18, '碘伏棉签',     '',            '',           5, 1, 17),
    (19, '脆脆鲨',       '',            '',           3, 1, 18),
    (20, '等渗能量胶',   '康比特',      '',           3, 1, 19),
    (21, '高渗能量胶',   'sis',         '',           3, 1, 19),
    (22, '宝矿力粉',     '',            '',           3, 1, 20),
    (23, '高反药',       '',            '',           3, 1, 21);

-- Seed activity
INSERT OR IGNORE INTO activities (id, name, icon) VALUES
    (1, '徒步', '🥾');

-- Seed activity_items (keep for backward compat)
INSERT OR IGNORE INTO activity_items (id, activity_id, item_id) VALUES
    (1,  1, 1),  (2,  1, 2),  (3,  1, 3),  (4,  1, 4),
    (5,  1, 5),  (6,  1, 6),  (7,  1, 7),  (8,  1, 8),
    (9,  1, 9),  (10, 1, 10), (11, 1, 11), (12, 1, 12),
    (13, 1, 13), (14, 1, 14), (15, 1, 15), (16, 1, 16),
    (17, 1, 17), (18, 1, 18), (19, 1, 19), (20, 1, 20),
    (21, 1, 21), (22, 1, 22), (23, 1, 23);

-- Seed activity_slots (23 slots for activity 1)
INSERT OR IGNORE INTO activity_slots (id, activity_id, slot_name, category_id, is_essential, default_qty, sort_order) VALUES
    (1,  1, '软壳衣',     1, 1, 1, 1),
    (2,  1, '软壳裤',     1, 1, 1, 2),
    (3,  1, '硬壳/雨衣',  1, 1, 1, 3),
    (4,  1, '速干衣',     1, 1, 2, 4),
    (5,  1, '抓绒',       1, 1, 1, 5),
    (6,  1, '羽绒',       1, 1, 1, 6),
    (7,  1, '袜子',       1, 1, 2, 7),
    (8,  1, '徒步鞋',     2, 1, 2, 8),
    (9,  1, '登山杖',     2, 1, 2, 9),
    (10, 1, '轻装包',     2, 1, 1, 10),
    (11, 1, '头灯',       2, 1, 1, 11),
    (12, 1, '墨镜',       2, 1, 1, 12),
    (13, 1, '防晒霜',     6, 1, 1, 13),
    (14, 1, '充电宝',     4, 1, 1, 14),
    (15, 1, '手表充电线', 4, 1, 1, 15),
    (16, 1, '手机充电线', 4, 1, 1, 16),
    (17, 1, '创口贴',     5, 1, 1, 17),
    (18, 1, '碘伏棉签',   5, 1, 1, 18),
    (19, 1, '零食',       3, 1, 1, 19),
    (20, 1, '能量胶(等渗)', 3, 1, 1, 20),
    (21, 1, '能量胶(高渗)', 3, 1, 1, 21),
    (22, 1, '电解质',     3, 1, 1, 22),
    (23, 1, '高反药',     3, 1, 1, 23);

-- Seed activity_slot_tags (each slot accepts its default item's tag)
INSERT OR IGNORE INTO activity_slot_tags (id, slot_id, tag_id) VALUES
    (1,  1,  1),
    (2,  2,  2),
    (3,  3,  3),
    (4,  4,  4),
    (5,  5,  5),
    (6,  6,  6),
    (7,  7,  7),
    (8,  8,  8),
    (9,  9,  9),
    (10, 10, 10),
    (11, 11, 11),
    (12, 12, 12),
    (13, 13, 13),
    (14, 14, 14),
    (15, 15, 15),
    (16, 16, 15),
    (17, 17, 16),
    (18, 18, 17),
    (19, 19, 18),
    (20, 20, 19),
    (21, 21, 19),
    (22, 22, 20),
    (23, 23, 21);

-- Seed tip
INSERT OR IGNORE INTO tips (id, activity_id, content) VALUES
    (1, 1, '所有穿身上的东西不要含棉麻');

-- Seed trip
INSERT OR IGNORE INTO trips (id, name, activity_id, start_date, end_date, status) VALUES
    (1, '五一哈巴西坡', 1, '2025-05-01', '2025-05-03', 'packing');

-- Seed trip_items (with slot_id)
INSERT OR IGNORE INTO trip_items (id, trip_id, item_id, qty, item_status, notes, slot_id) VALUES
    (1,  1, 1,  1, '',            '', 1),
    (2,  1, 2,  1, '',            '', 2),
    (3,  1, 3,  1, 'need_buy',    '', 3),
    (4,  1, 4,  2, '',            '', 4),
    (5,  1, 5,  1, '',            '', 5),
    (6,  1, 6,  1, 'need_buy',    '', 6),
    (7,  1, 7,  2, '',            '', 7),
    (8,  1, 8,  2, '',            '', 8),
    (9,  1, 9,  2, '',            '', 9),
    (10, 1, 10, 1, 'need_give',   '小微', 10),
    (11, 1, 11, 1, 'need_charge', '需寻找', 11),
    (12, 1, 12, 1, 'need_find',   '', 12),
    (13, 1, 13, 1, 'need_fetch',  '需去公司拿', 13),
    (14, 1, 14, 1, 'need_find',   '', 14),
    (15, 1, 15, 1, '',            '', 15),
    (16, 1, 16, 1, '',            '', 16),
    (17, 1, 17, 1, 'need_buy',    '', 17),
    (18, 1, 18, 1, 'need_find',   '', 18),
    (19, 1, 19, 1, '',            '', 19),
    (20, 1, 20, 1, '',            '', 20),
    (21, 1, 21, 1, '',            '', 21),
    (22, 1, 22, 1, '',            '', 22),
    (23, 1, 23, 1, '',            '', 23);

-- Incremental migration: add new columns (safe to re-run via ALTER TABLE tolerance in db.rs)
ALTER TABLE items ADD COLUMN tag_id INTEGER REFERENCES tags(id);
ALTER TABLE trip_items ADD COLUMN is_essential INTEGER NOT NULL DEFAULT 0;
ALTER TABLE trip_items ADD COLUMN slot_id INTEGER REFERENCES activity_slots(id)

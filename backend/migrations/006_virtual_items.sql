-- Virtual items: seed attribute definitions for item_type / expiry_date / file_url
-- This validates that the dynamic attribute system can express new item kinds without code changes.

INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order, is_identity)
VALUES ('item_type', '物品类型', 'text', '{"options":["实体","虚拟"]}', 13, 1);

INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order)
VALUES ('expiry_date', '过期日期', 'text', '{}', 14);

INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order)
VALUES ('file_url', '文件链接', 'text', '{}', 15);

ALTER TABLE attribute_definitions ADD COLUMN tag_scope TEXT NOT NULL DEFAULT '';

-- 订单相关属性定义
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('product_code', '货号', 'text', '{}', 20);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('sub_brand', '子品牌', 'text', '{}', 21);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('color', '颜色', 'text', '{}', 22);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('size', '尺码', 'text', '{}', 23);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('price', '价格', 'number', '{"suffix":"元"}', 24);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order) VALUES ('order_number', '订单号', 'text', '{}', 25);

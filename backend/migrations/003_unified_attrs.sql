-- Add new columns to attribute_definitions for unified attribute system
ALTER TABLE attribute_definitions ADD COLUMN is_identity INTEGER NOT NULL DEFAULT 0;
ALTER TABLE attribute_definitions ADD COLUMN is_required INTEGER NOT NULL DEFAULT 0;
ALTER TABLE attribute_definitions ADD COLUMN default_value TEXT NOT NULL DEFAULT '';
ALTER TABLE attribute_definitions ADD COLUMN search_weight INTEGER NOT NULL DEFAULT 0;

-- Update existing default_qty definition (already exists from 001_initial.sql)
UPDATE attribute_definitions SET is_required = 1, default_value = '1', search_weight = 0 WHERE key = 'default_qty';

-- Insert core attribute definitions for name/brand/model/notes
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order, is_identity, is_required, default_value, search_weight) VALUES ('name', '名称', 'text', '{}', 0, 1, 1, '', 10);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order, is_identity, is_required, default_value, search_weight) VALUES ('brand', '品牌', 'text', '{}', 0, 1, 0, '', 5);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order, is_identity, is_required, default_value, search_weight) VALUES ('model', '型号', 'text', '{}', 0, 1, 0, '', 3);
INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, sort_order, is_identity, is_required, default_value, search_weight) VALUES ('notes', '备注', 'text', '{}', 0, 0, 0, '', 1);

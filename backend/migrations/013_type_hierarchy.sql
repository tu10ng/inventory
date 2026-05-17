PRAGMA foreign_keys = OFF;

-- Rename child table columns first (FK references)
ALTER TABLE items RENAME COLUMN tag_id TO type_id;
ALTER TABLE attribute_definitions RENAME COLUMN tag_scope TO type_scope;

-- Rename junction table + column
ALTER TABLE activity_slot_tags RENAME TO activity_slot_types;
ALTER TABLE activity_slot_types RENAME COLUMN tag_id TO type_id;

-- Rename main table
ALTER TABLE tags RENAME TO types;

-- Add tree hierarchy support
ALTER TABLE types ADD COLUMN parent_id INTEGER REFERENCES types(id);
CREATE INDEX IF NOT EXISTS idx_types_parent_id ON types(parent_id);

PRAGMA foreign_keys = ON;

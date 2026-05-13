-- Activity includes: one activity can reference another
-- Unified model: no separate "module" concept — activities compose other activities.

CREATE TABLE IF NOT EXISTS activity_includes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    included_activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE(activity_id, included_activity_id)
);

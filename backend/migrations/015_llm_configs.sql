CREATE TABLE IF NOT EXISTS llm_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task TEXT NOT NULL UNIQUE,
    provider_name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    model TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO llm_configs (task, provider_name, base_url, api_key, model, is_active)
VALUES ('parse', 'DeepSeek', 'https://api.deepseek.com/v1', '', 'deepseek-chat', 1);

INSERT OR IGNORE INTO llm_configs (task, provider_name, base_url, api_key, model, is_active)
VALUES ('organize', 'DeepSeek', 'https://api.deepseek.com/v1', '', 'deepseek-chat', 1);

INSERT OR IGNORE INTO llm_configs (task, provider_name, base_url, api_key, model, is_active)
VALUES ('ocr', 'Kimi', 'https://api.moonshot.cn/v1', '', 'moonshot-v1-8k-vision', 1);

# Handlers — 路由处理函数

每个资源对应一个文件，`mod.rs` 负责注册所有路由到 Axum Router。

## 文件清单

| 文件 | 资源 | 端点 |
|------|------|------|
| `categories.rs` | 物品分类 | `GET/POST /api/categories`, `PUT/DELETE /api/categories/{id}` |
| `items.rs` | 物品库 | CRUD + `GET /api/item-stats`, `GET /api/item-stats/{id}` |
| `people.rs` | 人员 | `GET/POST /api/people`, `PUT/DELETE /api/people/{id}` |
| `activities.rs` | 活动模板 | CRUD + 子资源 items/tips |
| `trips.rs` | 行程 | CRUD + `populate` / `resync` / `clone` |
| `trip_items.rs` | 行程物品 | CRUD + `check` + `bulk_update` |
| `mod.rs` | 路由注册 | `pub fn router() -> Router<SqlitePool>` |

## 各 handler 函数

### categories.rs
- `list` / `create` / `update` / `delete` — 标准 CRUD

### items.rs
- `list` / `get` / `create` / `update` / `delete` — 标准 CRUD
- `usage_stats()` — `GET /api/item-stats`，返回 `Vec<ItemUsageCount>`（每个物品被多少行程使用）
- `usage_detail(id)` — `GET /api/item-stats/{id}`，返回 `ItemUsageStats`（物品关联的行程列表）

### people.rs
- `list` / `create` / `update` / `delete` — 标准 CRUD

### activities.rs
- `list` / `get` / `create` / `update` / `delete` — 活动 CRUD
- `list_items` / `add_item` / `remove_item` — 活动物品关联管理
- `list_tips` / `create_tip` / `update_tip` / `delete_tip` — 活动提示管理

### trips.rs
- `list` / `get` / `create` / `update` / `delete` — 行程 CRUD
- `populate(trip_id)` — 从关联活动模板填充物品（`INSERT OR IGNORE`，含 is_essential）
- `resync(trip_id)` — 同步模板：移除模板外物品（含所有手动添加项）+ 去重 + 补充新槽位
- `resync_preview(trip_id)` — 返回 `ResyncPreview`：将移除和新增的物品列表（含 reason），供前端确认
- `compute_resync_diff()` — 内部函数，计算 resync 差异，preview 和 resync 共用
- `clone(trip_id)` — 复制行程及所有物品，checked 重置为 false，名称加"(副本)"

### trip_items.rs
- `list(trip_id)` / `create(trip_id)` — 行程物品列表/添加
- `update(id)` / `delete(id)` — 更新/删除（update 支持部分字段）
- `check(id)` — PATCH 切换勾选状态
- `bulk_update(trip_id)` — PATCH 批量更新（checked / person_id / item_status）

## 添加新端点步骤

1. 在对应 handler 文件添加 `pub async fn`
2. 在 `models.rs` 添加请求/响应结构体
3. 在 `mod.rs` 的 `router()` 中注册路由
4. 注意：literal 路径必须放在 `{id}` 路径之前，或使用独立前缀避免歧义

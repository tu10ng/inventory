# Inventory — 出行物品清单管理系统

## 项目结构

pnpm monorepo，前后端分离。

```
├── backend/              Rust (Axum 0.8 + SQLite/sqlx 0.8)
│   ├── src/
│   │   ├── main.rs           入口，启动 0.0.0.0:3000
│   │   ├── db.rs             SQLite 连接池 + migration（ALTER TABLE 容错）
│   │   ├── error.rs          AppError：anyhow → axum 500 响应
│   │   ├── models.rs         数据模型 (serde + sqlx::FromRow)
│   │   └── handlers/         路由处理（每个资源一个文件）
│   └── migrations/
│       └── 001_initial.sql   建表 + 种子数据 + ALTER TABLE 增量迁移
├── frontend/             SvelteKit 2 + Svelte 5 (runes) + TypeScript
│   └── src/
│       ├── app.css               全局样式 + 物品栏暗色主题 + 打印样式
│       ├── lib/api/client.ts     fetch 封装，base path /api
│       ├── lib/types/index.ts    TS 类型定义
│       ├── lib/utils/status.ts   状态标签/选项常量
│       ├── lib/components/       11 个可复用 Svelte 组件
│       └── routes/               页面路由
│           ├── +page.svelte          首页（最近行程快速入口）
│           ├── trips/+page.svelte    行程列表（含克隆）
│           ├── trips/[id]/           行程详情：双栏布局（清单 + 物品库）
│           ├── activities/           活动模板管理（含 is_essential 标记）
│           └── items/                物品库（搜索/筛选/网格-列表切换/使用统计）
├── dev.sh                一键启动前后端
└── something.txt         原始出行清单（org-mode，已导入为种子数据）
```

## 开发命令

```bash
./dev.sh                          # 一键启动（推荐）
cd backend && cargo run           # API :3000
cd frontend && pnpm dev           # Dev :5173（proxy /api → :3000）
cd frontend && pnpm check         # TS + Svelte 类型检查
cd backend && cargo build         # 后端编译检查
```

## 数据库

- SQLite，文件 `backend/inventory.db`，WAL 模式
- Migration 在 `001_initial.sql` 一个文件里，启动时自动执行
- 添加新表/种子数据：直接追加到 `001_initial.sql` 末尾，用 `INSERT OR IGNORE` + 显式 id 保证幂等
- 增量迁移用 `ALTER TABLE ADD COLUMN`，`db.rs` 会自动忽略 "duplicate column name" 错误
- 重置数据库：删除 `backend/inventory.db*` 后重启

## 数据模型

| 表 | 说明 |
|---|---|
| categories | 物品分类（服装/装备/营养/电子/急救/洗漱/证件/其他） |
| items | 物品库（name, brand, model, category_id, default_qty） |
| activities | 活动模板（如"徒步"） |
| activity_items | 活动↔物品关联（含 is_essential） |
| tips | 活动小贴士 |
| people | 人员 |
| trips | 行程（name, activity_id, start/end_date, status） |
| trip_items | 行程物品（qty, checked, item_status, notes, person_id, is_essential） |

trip status: `planning` → `packing` → `done`

item_status: `''` | `need_buy` | `need_find` | `need_charge` | `need_fetch` | `need_give`

## API 路径约定

RESTful，前缀 `/api`。

| 方法 | 路径 | 说明 |
|------|------|------|
| GET/POST | `/api/trips` | 行程列表/创建 |
| GET/PUT/DELETE | `/api/trips/{id}` | 行程 CRUD |
| GET/POST | `/api/trips/{id}/items` | 行程物品 |
| PUT/DELETE | `/api/trip-items/{id}` | 更新/删除行程物品（注意连字符） |
| PATCH | `/api/trip-items/{id}/check` | 切换勾选 |
| PATCH | `/api/trips/{id}/items/bulk` | 批量更新行程物品 |
| POST | `/api/trips/{id}/populate` | 从模板填充物品 |
| POST | `/api/trips/{id}/resync` | 从模板同步（移除模板外物品+补充新槽位） |
| POST | `/api/trips/{id}/resync-preview` | 同步预览（返回将移除/新增的物品列表） |
| POST | `/api/trips/{id}/clone` | 克隆行程 |
| GET | `/api/item-stats` | 物品使用统计（独立路径，避免 `{id}` 冲突） |
| GET | `/api/item-stats/{id}` | 单个物品使用详情 |

## 注意事项

- Migration 用 `split(';')` 切分 SQL，**字符串值中不能含分号**
- 前端 Vite dev 配置了 `/api` 反向代理到 `:3000`
- 后端 CORS 全开（开发模式）
- Svelte 5 runes 模式：所有组件使用 `$state`/`$derived`/`$effect`/`$props`
- Axum 0.8 路径参数用 `{id}` 语法（非 `:id`），**literal 路径要避免与 `{id}` 歧义**（已踩坑：`/items/stats` 被 `/items/{id}` 抢匹配，改为 `/item-stats`）
- 前端 adapter-static + SPA fallback，`ssr = false`

## 业务逻辑设计教训

以下问题在初版中被遗漏，根因是**只关注了数据/API 的技术正确性，忽视了用户操作流程中的防错设计和信息透明度**：

1. **添加物品无查重**：把"添加"当作纯 CRUD 实现，没有从用户真实场景出发——用户可能忘记已经加过，或从物品库面板重复点击。教训：**任何"新增"操作都应检查是否与已有数据冲突，至少给予提醒**。
2. **resync 逻辑散乱**：最初只处理"新增模板槽位"，后来逐步补丁式追加删除逻辑（5 个独立 DELETE），每次只修一个 edge case。没有退一步从"resync 的完整行为定义"出发重新设计。教训：**当一个函数被反复追加补丁时，应该停下来重新定义它的整体行为，而不是继续堆叠**。
3. **同步无预览**：开发者知道 resync 会做什么所以觉得一键执行合理，但用户不知道——可能丢失手动添加的物品却毫无察觉。教训：**任何批量删除/修改操作，必须先展示将发生的变化让用户确认（preview → confirm 模式）**。

总结：实现功能时不能只想"API 入参出参对不对"，要**模拟用户的完整操作路径**，在每个有副作用的步骤问自己：用户知道会发生什么吗？操作可逆吗？有没有防误触？

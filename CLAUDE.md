# Inventory — 出行物品清单管理系统

## 项目结构

pnpm monorepo，前后端分离。

```
├── backend/          Rust (Axum + SQLite/sqlx)
│   ├── src/
│   │   ├── main.rs          入口，启动 0.0.0.0:3000
│   │   ├── db.rs            SQLite 连接池 + migration
│   │   ├── models/          数据模型 (serde Serialize/Deserialize)
│   │   └── handlers/        路由处理 (categories, items, activities, trips, trip_items, people)
│   └── migrations/
│       └── 001_initial.sql  建表 + 种子数据（单文件，include_str! 加载，split(';') 逐条执行）
├── frontend/         SvelteKit 2 + TypeScript
│   └── src/
│       ├── lib/api/client.ts    fetch 封装，base path /api
│       ├── lib/types/index.ts   TS 类型定义
│       └── routes/              页面路由
│           ├── trips/[id]/      行程详情/打包清单
│           ├── activities/      活动模板管理
│           └── items/           物品库管理
├── dev.sh            一键启动前后端
└── something.txt     原始出行清单（org-mode，已导入为种子数据）
```

## 开发命令

```bash
# 一键启动（推荐）
./dev.sh

# 分别启动
cd backend && cargo run          # API :3000
cd frontend && pnpm dev          # Dev :5173（proxy /api → :3000）

# 构建
cd frontend && pnpm build
cd backend && cargo build --release
```

## 数据库

- SQLite，文件 `backend/inventory.db`，WAL 模式
- Migration 在 `001_initial.sql` 一个文件里，启动时自动执行
- 添加新表/种子数据：直接追加到 `001_initial.sql` 末尾，用 `INSERT OR IGNORE` + 显式 id 保证幂等
- 重置数据库：删除 `backend/inventory.db*` 后重启

## 数据模型

| 表 | 说明 |
|---|---|
| categories | 物品分类（服装/装备/营养/电子/急救/洗漱/证件/其他） |
| items | 物品库（name, brand, model, category_id, default_qty） |
| activities | 活动模板（如"徒步"） |
| activity_items | 活动↔物品关联 |
| tips | 活动小贴士 |
| people | 人员 |
| trips | 行程（name, activity_id, start/end_date, status） |
| trip_items | 行程物品（qty, checked, item_status, notes, person_id） |

trip status: `planning` → `packing` → `done`

item_status: `''` | `need_buy` | `need_find` | `need_charge` | `need_fetch` | `need_give`

## API 路径约定

RESTful，前缀 `/api`。示例：

- `GET/POST /api/trips`
- `GET/PUT/DELETE /api/trips/{id}`
- `GET/POST /api/trips/{id}/items`
- `PUT/DELETE /api/trip-items/{id}`（注意用连字符）
- `PATCH /api/trip-items/{id}/check`
- `POST /api/trips/{id}/populate` — 从活动模板填充物品

## 注意事项

- Migration 用 `split(';')` 切分 SQL，**字符串值中不能含分号**
- 前端 Vite dev 配置了 `/api` 反向代理到 `:3000`
- 后端 CORS 全开（开发模式）

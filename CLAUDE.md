# Inventory — 出行物品清单管理系统

## *** 必须使用中文思考 ***

## *** 项目的关键技术决策要提供选项, 让用户选择 ***

## *** 每次修改问题后, 都要增加一个"反思"的步骤, 反思为什么之前会做错, 并且一定要把复盘原因写入CLAUDE.md, 避免下次重犯 ***

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
│       ├── 001_initial.sql   建表 + 种子数据
│       └── 002_tag_scope.sql  tag_scope 列迁移
├── frontend/             SvelteKit 2 + Svelte 5 (runes) + TypeScript
│   └── src/
│       ├── app.css               全局样式 + 物品栏暗色主题 + 打印样式
│       ├── lib/api/client.ts     fetch 封装，base path /api
│       ├── lib/types/index.ts    TS 类型定义
│       ├── lib/utils/status.ts   状态标签/选项常量（动态 API 加载）
│       ├── lib/utils/columns.ts   列定义（动态加载）
│       ├── lib/utils/cellValue.ts  单元格值提取
│       ├── lib/utils/itemFilters.ts 物品筛选/排序
│       ├── lib/components/       18 个可复用 Svelte 组件
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
- Migration 文件在 `migrations/` 目录，按文件名排序执行
- `_migrations` 追踪表防止重复执行
- `db.rs` 扫描目录，执行未追踪的 migration 文件
- 仍使用 `split(';')` 切分 SQL — **字符串值不能含分号**
- 重置数据库：删除 `backend/inventory.db*` 后重启

## 数据模型

| 表 | 说明 |
|---|---|
| categories | 物品分类（服装/装备/营养/电子/急救/洗漱/证件/其他） |
| tags | 物品子类型标签，每个 tag 归属一个 category |
| items | 物品库（name, brand, model, category_id, default_qty, notes, tag_id, attrs） |
| attribute_definitions | 物品属性定义（key/label/attr_type/config/category_scope/tag_scope） |
| status_definitions | 状态定义（scope: item/trip） |
| activities | 活动模板（如"徒步"） |
| activity_slots | 活动槽位（slot_name, category_id, tags, default_item_id, is_essential） |
| activity_slot_tags | 槽位↔标签多对多关联 |
| tips | 活动小贴士 |
| people | 人员 |
| trips | 行程（name, activity_id, start/end_date, status） |
| trip_items | 行程物品（qty, checked, item_status, notes, person_id, is_essential, slot_id） |
| _migrations | 迁移追踪表 |

trip status: `planning` → `packing` → `done`

item_status 动态定义于 `status_definitions` 表（scope: item）

## API 路径约定

RESTful，前缀 `/api`。

| 方法 | 路径 | 说明 |
|------|------|------|
| GET/POST | `/api/categories` | 分类列表/创建 |
| PUT/DELETE | `/api/categories/{id}` | 更新/删除分类 |
| GET/POST | `/api/tags` | 标签列表/创建 |
| PUT/DELETE | `/api/tags/{id}` | 更新/删除标签 |
| GET/POST | `/api/items` | 物品列表/创建 |
| GET/PUT/DELETE | `/api/items/{id}` | 物品 CRUD |
| GET | `/api/item-stats` | 物品使用统计 |
| GET | `/api/item-stats/{id}` | 单个物品使用详情 |
| POST | `/api/items/export` | 导出物品库 JSON |
| POST | `/api/items/import-preview` | 导入预览 |
| POST | `/api/items/import` | 执行导入 |
| GET/POST | `/api/attribute-definitions` | 属性定义 CRUD |
| PUT/DELETE | `/api/attribute-definitions/{id}` | 更新/删除属性定义 |
| GET/POST | `/api/status-definitions` | 状态定义 CRUD |
| PUT/DELETE | `/api/status-definitions/{id}` | 更新/删除状态定义 |
| GET/POST | `/api/people` | 人员列表/创建 |
| PUT/DELETE | `/api/people/{id}` | 更新/删除人员 |
| GET/POST | `/api/activities` | 活动列表/创建 |
| PUT/DELETE | `/api/activities/{id}` | 活动 CRUD |
| GET/POST | `/api/activities/{id}/slots` | 活动槽位 |
| PUT/DELETE | `/api/activity-slots/{id}` | 更新/删除槽位 |
| GET/POST | `/api/activities/{id}/tips` | 活动小贴士 |
| PUT/DELETE | `/api/tips/{id}` | 更新/删除贴士 |
| GET/POST | `/api/trips` | 行程列表/创建 |
| GET/PUT/DELETE | `/api/trips/{id}` | 行程 CRUD |
| GET/POST | `/api/trips/{id}/items` | 行程物品 |
| GET | `/api/trips/{id}/items/enriched` | 行程物品（含 slot 信息+候选物品） |
| PUT/DELETE | `/api/trip-items/{id}` | 更新/删除行程物品 |
| PATCH | `/api/trip-items/{id}/check` | 切换勾选 |
| POST | `/api/trip-items/{id}/save-as-slot` | 将行程物品保存为模板槽位 |
| PATCH | `/api/trips/{id}/items/bulk` | 批量更新 |
| POST | `/api/trips/{id}/populate` | 从模板填充 |
| POST | `/api/trips/{id}/resync` | 从模板同步 |
| POST | `/api/trips/{id}/resync-preview` | 同步预览 |
| POST | `/api/trips/{id}/clone` | 克隆行程 |
| POST | `/api/ai/parse-items` | AI 物品解析（阻塞式） |
| POST | `/api/ai/parse-items-stream` | AI 物品解析（SSE 流式） |
| POST | `/api/ai/organize-preview` | AI 整理预览 |
| POST | `/api/ai/organize-apply` | AI 整理执行 |
| POST | `/api/ai/ocr` | OCR 图片识别 |

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

4. **AI prompt 未适配新数据源**：OCR 功能实现后，LLM 的 system prompt 仍是"户外装备专家"语境（仅适用自然语言描述物品），不知道如何处理订单号、价格单位转换（"分"→"元"）、子品牌与母品牌关系（QUECHUA→迪卡侬）等订单特有的语义。教训：**当新增上游数据源时，必须检查整个 pipeline 每一环是否适配——不只看数据产出格式，还要看下游消费者（LLM）的 prompt 是否给了足够的上下文**。

## 流式 AI 解析实现笔记

### SSE 流式传输架构
- `SseEvent` enum（`#[serde(tag = "type")]`）区分 thinking/progress/result/error
- 后端通过 `tokio::sync::mpsc::unbounded_channel` 在 spawned task 和 SSE stream 之间传递事件
- `UnboundedReceiverStream`（tokio-stream）将 receiver 转为 futures Stream 供 Axum Sse 使用
- 前端通过 `fetch()` + `ReadableStream.getReader()` 消费 SSE

### 注意事项
- OpenAI streaming 模式（`stream: true`）与 `response_format: json_object` 不兼容
- 流式模式下，prompt 需要引导 LLM 先输出思考文字再输出 JSON（`---JSON---` 分隔符）
- `reqwest::Response::bytes_stream()` 实现 `futures_core::Stream`，可直接用于流式读取
- `response.json()` 失败后不能再用 `response.text()`，body stream 已被消费。应先用 `response.text()` 读文本，再 `JSON.parse()` 尝试解析
- Svelte 5 模板中 `{...}` 内不能使用 `{ ...; ... }` body 语法（大括号冲突），应抽取为 `<script>` 中的函数
- `AppError` 不实现 `Display`，format 时需用 `{:?}` / `{:#?}`（需 `#[derive(Debug)]`）

## 2026-05-12 代码质量修复复盘

### P0 Bug 根因

1. **`tag_scope` 列遗漏**：`002_tag_scope.sql` 迁移加了列，但只更新了 `attributes.rs` 的 CRUD handler，export/import 是后来加的功能，被遗漏。教训：**Schema 变更时应该全局 grep 所有引用该表的 SQL**（`grep "attribute_definitions"` 确认所有引用点）。

2. **`organize_apply` 的 `new_tags` 永远为空**：`OrganizeApplyResponse` 从 `AiParseResponse`（有 `new_tags`）照搬，但 apply 阶段 tag 已在 preview 阶段创建完毕。教训：**复制代码时必须审视每个字段在目标场景下是否还有意义，照搬结构体 ≠ 照搬逻辑**。

### P1 死代码根因

- `svelte-dnd-action` 是 slot+tag 系统开发时预装的依赖，最终用了原生 HTML5 drag，但没有清理。教训：**依赖不应该"预装"，应该在真正需要时才加，加了不用就要删**。

### P2 复杂度问题根因

- `compute_resync_diff` 的 N+1 查询是逐步堆叠补丁的典型后果：最初只处理"新增槽位"，后来追加删除逻辑（3 个独立 if 分支），每次只在循环体内加分支，没人退一步想"这条 item name 可以批量查"。教训：**当一个函数内部出现 3 个以上结构相似的代码块时，就该考虑提取公共逻辑**。

### P3 前端质量问题根因

- **"动态化改造不彻底"** 是最深层的问题：status 改为动态加载了，但 UI 颜色还在用硬编码 CSS；columns 动态加载了，但 `cellValue.ts` 只覆盖了 2/4 个核心字段。这类"改一半"的问题比不做更危险——给人"已支持动态化"的假象。教训：**做动态化改造时，必须全链路追踪：API → 类型 → 工具函数 → 组件渲染，每个环节都要确认不再有硬编码**。

### P4 文档腐烂根因

- 三阶段架构重构后（分类用户化→状态动态化→属性动态化），没有人系统地更新文档。大型重构的 checklist 必须包含"更新所有相关 CLAUDE.md"。

## 2026-05-12 统一属性系统实施复盘

### 实施中踩的坑

1. **table-rebuild 模式在 items 表上失败**：`rebuild_trip_items_fk` 和 `rebuild_trips_table` 都成功使用 CREATE→INSERT→DROP→RENAME 模式重建表，但 items 表有多个子表（trip_items、activity_slots）的 FK 引用。即使 `PRAGMA foreign_keys = OFF`，SQLite 仍拒绝 DROP TABLE items（错误信息不明确，只说 RENAME 时 items 已存在）。教训：**table-rebuild 模式在父表（被 FK 引用的表）上不可行，应改用 in-place UPDATE 迁移**。

2. **NOT NULL 约束的遗留列**：旧表 `name` 列有 NOT NULL 约束，新的 INSERT 只提供 `(category_id, tag_id, attrs)` 不提供 `name`，导致 INSERT 失败。教训：**Schema 迁移不能只改代码的 SELECT/INSERT，还要检查所有列的约束条件。旧列不清除时，约束仍然生效**。

3. **UPDATE 的 attrs 替换 vs 合并**：最初的 update handler 直接用 `body.attrs.unwrap_or(existing.attrs)` 替换整个 attrs，导致 partial update 时丢失其他字段。教训：**JSON 列的部分更新语义应该是 merge 而不是 replace，与关系列（category_id）的语义不同**。

### 最佳实践总结

- Table rebuild 只适用于叶子表（无 FK 子表）；父表迁移用 in-place UPDATE + 旧列留空
- JSON 列更新：前端发送全量 attrs（通过 `{ ...existing, [field]: value }`），后端 merge 到 existing attrs（前端视图优先）
- 旧物理列作为 shadow copy 保留：INSERT/UPDATE 时从 attrs JSON 提取值同步写入（NOT NULL 兼容）

## 2026-05-13 自动化测试实施复盘

### 测试架构决策

1. **后端：sqlite::memory: + 每测试独立池**：最初计划用"事务 + rollback"模式，但 handler 函数接收 `State<SqlitePool>` 直接操作池，不在事务内。改为每测试调用 `init_test_pool()` 创建独立的 `:memory:` 数据库，天然隔离，无需清理。

2. **前端：vitest 组件测试需要 `resolve.conditions: ['browser']`**：`@testing-library/svelte` v5 默认走 Svelte 的 server 端渲染路径，`mount()` 不可用。添加 `resolve.conditions: ['browser']` 强制走浏览器端。

### 测试中发现的问题

- **没有种子 tag 数据**：`update_category_change` 测试使用了 `tag_id: Some(1)`，但 migrations 只 seed 了 categories 和 status_definitions/attribute_definitions，没有 seed tags。测试需要手动插入 tag 或使用不存在的 tag id。教训：**测试数据不能假设生产环境的种子数据完整，要显式准备或检查**。

### 前端组件测试的陷阱

- **Svelte 5 runes 组件 + jsdom 兼容**：Svelte 5 的 `$derived`/`$state` 在 jsdom 环境下的行为与浏览器一致，但需要正确的 vitest 配置。
- **`getByText` 对复合文本无效**：`ItemCard` 中 brand + model 渲染为 "始祖鸟 Beta LT"，不能单独 `getByText('始祖鸟')`，应匹配完整文本或按 class 查找。
- **`getByText` 对多处出现的文本**：`ItemDetailPanel` 中物品名称在 header 和动态属性区都出现，`getByText` 会报多个匹配，应用 `getAllByText`。
- **编辑模式检测依赖 `item.id`**：`ItemForm` 通过 `!!item?.id` 判断 isEdit，测试中传 `item: { id: 1, ... }` 才能触发"更新"模式。

### 测试命令

```bash
cd backend && cargo test       # 45 个后端测试，< 0.1s
cd frontend && pnpm test       # 48 个前端测试，< 1s
```

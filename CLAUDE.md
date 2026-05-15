# Inventory — 出行物品清单管理系统

## *** 必须使用中文思考 ***

## *** 项目的关键技术决策要提供选项, 让用户选择 ***

## *** 每次修改问题后, 都要增加一个"反思"的步骤, 反思为什么之前会做错, 并且一定要把复盘原因写入CLAUDE.md, 避免下次重犯 ***

## *** 不要删除数据库, 要测试请写测试用例 ***

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
cd backend && cargo test       # 53 个后端测试，< 0.2s
cd frontend && pnpm test       # 48 个前端测试，< 2s
```

## 2026-05-13 ItemDetailPanel 编辑 Bug 修复复盘

### Bug 1: `handleFieldUpdate` 对 `attrs` 字段双重包裹

**根因**：`handleFieldUpdate` 的原始逻辑只有两个分支：top-level 字段（`category_id`/`tag_id`）直接写入，其他字段都视为 attrs 内的子字段，用 `{ ...attrs, [field]: value }` 包裹。但当 `ItemDetailPanel.updateAttr()` 调用时，`field` 就是 `'attrs'`，`value` 已经是完整的 attrs 对象（在 `updateAttr` 中已做过 `{ ...item.attrs, [key]: value }` 合并）。`handleFieldUpdate` 又做了一次 `{ ...attrs, attrs: value }`，导致后端收到 `{ attrs: { name: "旧", attrs: { name: "新" } } }`，旧值覆盖新值。

**教训**：**当调用方已经做了数据组装（`updateAttr` 构造完整 attrs），接收方不应再二次组装**。函数签名 `handleFieldUpdate(field: string, value: unknown)` 的 `field` 参数有两层语义：它既是"要修改的字段名"，也是"数据已经按什么层级组装好了"的提示。当 `field === 'attrs'` 时，`value` 已经是最终数据，不应再嵌套。这类"透传已组装数据"的模式应该有明确的短路分支。

### Bug 2: `text` 类型已知属性用了药丸编辑器

**根因**：属性编辑器选择逻辑只有 `text && config.options` 分支使用 `InlineEditPills`，其余类型（weight/number/bar/stars/bool 等）都有独立分支，但纯 `text` 类型（无 options）落入 `{:else}` 兜底，用了 `InlineEditPills` + `freeform={true}`。这个兜底逻辑的意图是处理"未知的 ad-hoc 属性类型"，但没有考虑到 `text` 类型是已知类型，只是恰好没有 options。

**教训**：**`{:else}` 兜底分支的语义应该是"真正未知的情况"，而不是"我还没处理的已知情况"**。在写 if-else 链时，每增加一种已知的 `attr_type`，都应该显式处理，让 `{:else}` 只覆盖真正的未知类型。否则已定义的类型会被迫走不适合的 UI 控件。

## 2026-05-13 元层能力扩展实施复盘

### 实施内容

五个 Phase 全部完成：
1. **虚拟物品**（纯配置验证）：新增 3 个 attribute_definitions 种子数据（item_type/expiry_date/file_url），前端 ItemForm/ItemDetailPanel 加入 item_type 感知的 $derived 过滤
2. **动态关系系统**（新元层）：relation_types + item_relations 表/模型/Handler/路由/前端类型/ItemDetailPanel「关联物品」section
3. **display_rules 聚合视图**（扩展元层）：DisplayRuleConfig 结构体（mode/summary_fields），前端 summary 模式渲染（分组卡片显示汇总字段值）
4. **活动互相引用**（重构）：activity_includes 表，collect_activity_slots() 递归展开 + 循环检测，populate/computer_resync_diff 共用
5. **物品批量操作**（UX 改进）：POST /api/items/batch，BulkActionBar 泛化为 actions prop

### 反思

1. **种子数据数量变更必须同步更新测试断言**：新增第 4 条 display_rules 种子后，`list_returns_seed_rules` 和 `create_and_list` 测试的 expected count 需要 +1。教训：**修改种子数据后，全局 grep 对应的 `assert_eq!(rules.len()` 或硬编码数量**。

2. **泛化组件要同步检查所有调用方**：BulkActionBar 从 trip_items 专用改为通用后，ChecklistPanel 是唯一调用方。教训：**组件 API 变更后，用 grep 找到所有 `<ComponentName` 引用点逐一更新**。

3. **migration 的 split(';') 限制需要记住**：relation_types 种子数据的 `'搭配'` label 不含分号所以安全，但任何含分号的字符串值（如 JSON 字符串内嵌分号）都会导致 migration 失败。教训：**种子数据中的 JSON 字段（如 display_rules.config）需要用单引号包裹或在 migration 外处理**。

4. **routes 注册顺序仍会踩坑**：`/api/items/batch` 必须放在 `/api/items/{id}` 之前，否则 `batch` 会被当作 `{id}` 解析。虽然本次没有重复踩坑（已注意），但这类 Axum 路由歧义是持久性陷阱。

5. **Rust 的 `use` 语句放在文件中间可行但不推荐**：最初在 items.rs 中间加了 `use crate::models::{BatchItemsRequest, BatchItemsResponse};`，与顶部的 import 冲突导致 "defined multiple times" 错误。应统一在文件顶部导入。

## 2026-05-13 补充缺失 UI 入口实施复盘

### 实施内容

1. **设置页新增 relation_types 管理 section**：遵循与 Categories 相同的 pattern（list + inline form），字段为 name/label/color/icon/bidirectional/sort_order
2. **物品库页新增批量操作**：复用已泛化的 BulkActionBar，支持批量删除和更改分类
3. **ItemListTable 新增 checkbox 列**：selectable 模式下每行前显示 checkbox，header 有全选/取消全选

### 反思

1. **组件有多条渲染路径时必须逐一检查**：ItemListTable 有 3 种 item 渲染路径（分组模式下的分组内容、分组模式下的未分组项、非分组模式），新增 checkbox 列时每条路径都要加上 `{#if selectable}` 块，容易遗漏。教训：**改动组件中重复出现的渲染代码块时，先用 grep 搜索所有出现位置，确认总数，然后逐一修改并确认没有遗漏**。

2. **批量操作后要重置 UI 状态**：batchDelete 和 batchChangeCategory 成功后不仅要清空 selectedItemIds，还要退出 selectable 模式并清空右侧面板（selectedItem/panelMode），否则 UI 会处于不一致状态（如右侧仍显示已删除物品的详情）。教训：**副作用操作（删除/修改）后，要追溯所有可能受影响的 UI 状态变量并重置**。

## 2026-05-14 物品数量支持 0 + 删除物品消除页面闪烁复盘

### 实施内容

1. **default_qty 验证放宽**：后端 models.rs 和 handlers/items.rs 的 `qty < 1` → `qty < 0`，错误信息改为"默认数量不能为负数"
2. **import/export 中去掉 `.max(1)` 夹持**：`attr_i64("default_qty").max(1)` 会错误地将 0 夹持为 1，改用直接 JSON 访问
3. **前端 InlineEdit min={0}**：ItemDetailPanel 数量编辑允许 0
4. **ItemCard 显示 x0**：`displayQty > 1 || displayQty === 0` 时显示徽章，0 时加 `.zero` 灰色半透明样式
5. **删除/批量操作乐观本地更新**：handleDelete/batchDelete/batchChangeCategory/handleSave 改为直接修改本地 `$state items` 数组，不再调用 `await load()` 触发全页重载

### 反思

1. **辅助方法的默认值可能遮蔽合法值**：`attr_i64("default_qty")` 在 key 不存在时返回 0，但 0 恰好是本次要支持的合法值。原始代码加 `.max(1)` 就是为了修正这个"错误的默认值"，但这同时夹持了真正的 0。教训：**辅助方法（如 `attr_i64`）的默认值应该是调用方显式选择的，而不是硬编码在方法内**。当默认值与某个合法值冲突时，应该用直接访问替代辅助方法。

2. **`await load()` 是全页重载的反模式**：delete/save 操作后调用 `await load()` 会设置 `loading = true`，导致整个页面被"加载中..."替换，数据返回后再渲染回来，造成视觉闪烁。教训：**CRUD 操作后应优先使用乐观本地更新（修改 $state 变量），只在确实需要重新计算服务端派生数据时才全量 reload**。`items` 是 `$state` 变量，修改后 `$derived` 链（filteredItems/sortedItems/groupedData）会自动重新计算，UI 无缝更新。

3. **BatchDelete 和 batchChangeCategory 需要在 API 调用前捕获 ID 快照**：在 `await api.post(...)` 之前捕获 `const idsToDelete = new Set(selectedItemIds)`，因为在 API 调用完成后立即清空 `selectedItemIds`。如果在 API 调用后仍引用 `selectedItemIds`，乐观更新时会拿到空集合。教训：**乐观更新中，如果副作用数据需要在异步操作后使用，必须在异步调用前捕获快照**。

## 2026-05-14 批量操作重构复盘

### 实施内容

1. 后端 `POST /api/items/batch` 的 `update` action 扩展支持 `tag_id`（含设 null）和 `attrs` merge（逐 item + transaction + legacy 列同步）
2. ItemListTable 复选框始终可见（移除 `{#if selectable}` 条件，prop 默认 `true`）
3. ItemGroupBlock 新增 `selectedIds`/`onToggleSelect` props，增加 checkbox 列
4. BulkActionBar 重写为属性驱动批量编辑器：属性下拉 → 按类型渲染值编辑器 → 应用/批量删除，同时保留 `actions` prop 向后兼容 ChecklistPanel
5. items/+page.svelte 移除 `selectable` 切换状态和 `toggleSelectMode`/`batchActions`，新增 `handleBatchUpdateAttr` 和 `batchAttrOptions`

### 反思

1. **修改共享组件时必须全局检查所有调用方**：BulkActionBar 被两个页面引用——`items/+page.svelte`（属性驱动模式）和 `ChecklistPanel.svelte`（legacy `actions` 模式）。重写时只考虑了 items 页的需求，删除了 `actions` prop，导致 ChecklistPanel 类型错误。教训：**组件 API 变更后，用 `grep "<ComponentName"` 找到所有引用点逐一检查，不能假设只有一个调用方**。向后兼容可以用可选 prop 共存——`actions` + `attrOptions` 两个模式互斥。

2. **Svelte 5 `bind:` 与 `unknown` 类型不兼容**：`editingValue: unknown` 作为状态变量，不能直接用 `bind:value` 或 `bind:checked`，需要改用 `value`/`checked` + `oninput`/`onchange` 显式 event handler 并做类型断言。

## 2026-05-14 通用 Excel 导入功能实施复盘

### 实施内容

后端新增 `handlers/excel.rs`，提供两个端点：
1. `POST /api/import/excel-preview` — 接收 multipart xlsx 文件，用 calamine 解析，返回 `ExcelPreviewResponse`（headers + rows，无业务逻辑）
2. `POST /api/import/excel-ai-stream` — 接收 headers + rows，动态构建 prompt，流式调用 LLM 解析为 `AiParsedItem[]` + `new_attr_defs`

前端新增 `ExcelImportModal.svelte` 组件，8 阶段流程：
- **AI 路径**: upload → parsing → preview-raw → ai-streaming → ai-preview → importing → done
- **手动映射路径**: upload → parsing → preview-raw → manual-mapping → importing → done

### 关键设计决策

1. **后端不做业务判断**：calamine 只做 xlsx → `{headers, rows}` 原样返回，列语义理解完全交给 LLM 或用户。这保证了系统对任意格式 Excel 的通用性。
2. **Prompt 动态构建**：分类体系、标签列表、属性定义都从 DB 实时查询后嵌入 prompt，而非硬编码。
3. **两种导入路径共存**：AI 智能模式（适合列名不规范）和手动映射模式（适合列名已规范），用户可选择。
4. **人工可在 AI 预览阶段修正**：AI 解析结果以可编辑表格形式展示，支持 InlineEdit 修改 name/brand/model/default_qty，以及下拉选择 category/tag。

### 反思

1. **calamine 的 trait bound 需要显式导入**：`open_workbook_from_rs` 返回泛型 `Xlsx<RS>`，必须 `use calamine::Reader;` 才能调用 `sheet_names()` 和 `worksheet_range()`。教训：**第三方 crate 使用时先确认 trait 是否需要手动导入**。

2. **calamine 的行迭代器类型无法推断**：`range.rows()` 返回的迭代器 item 类型是 `&[Data]`，但编译器无法自动推断，需要显式 `let first_row: Option<&[calamine::Data]>` 标注。教训：**当第三方库的迭代器类型推断失败时，直接给变量标注具体类型**。

3. **SSE 事件类型需要考虑前端 switch**：新增了 `ExcelResult` SseEvent 变体（含 `new_attr_defs` 字段），与现有 `Result` 变体（无 `new_attr_defs`）区分。**但前端的 SSE switch-case 没有 `excel_result` 分支**，导致 `ExcelResult` 事件被静默丢弃——Excel AI 流式解析的实际结果从未到达前端（反思中才发现）。教训：**新增 SSE 事件类型时必须同时检查前端 switch-case 是否覆盖了新类型，不能假设"新增类型会自动被处理"**。

4. **InlineEdit oncommit 类型是 `(val: string | number) => void`**：不管 type 是 text 还是 number，oncommit 都接受 `string | number`。对于 text 类型需要显式 `String(v)` 转换。

5. **items 行级逐个 POST 导入是保守选择**：目前采用逐行 POST `/api/items`，对大批量（500+）较慢但可见进度。后续可优化为 `POST /api/items/batch` 批量创建。

6. **可选回调函数调用前需判空**：`onBatchDelete` 和 `onBatchUpdateAttr` 在 props 中是可选类型（`?: () => Promise<void>`），调用前需要 `if (!onBatchDelete) return;` 守卫。

## 2026-05-14 统一 AI 解析流程复盘

### 实施内容

将 Excel AI 解析路径统一到文本 AI 端点：
- `SseEvent::Result` 增加 `new_attr_defs` 字段（`#[serde(default)]`），删除 `ExcelResult` 变体
- `parse_items_stream` 增加 `extract_new_attr_defs_from_text()` + INSERT OR IGNORE 到 attribute_definitions 表
- `build_system_prompt()` 增加 Excel 表格数据（"列名: 值 | 列名: 值" 格式）处理规则
- 删除 `excel.rs` 的 `build_excel_prompt()`、`excel_ai_stream()`、`extract_new_attr_defs_from_text()`
- 前端 `ExcelImportModal` 删除 AI 阶段（ai-streaming/ai-preview），AI 路径改为格式化 Excel 为文本 → 调用 `onOpenAiModal` → 复用 `AiAddModal`
- `AiAddModal` 新增 `onNewAttrs` prop

### 改造后流程
```
Excel:  上传 xlsx → 预览表格 ─┬─ [AI 解析] → 转文本 → AiAddModal → 确认 → 逐行创建
                              └─ [手动映射] → 列映射 → 预览 → 导入
OCR:    上传图片 → OCR 文本 ───→ AiAddModal → 确认 → 逐行创建
文本:   输入文字 ────────────→ AiAddModal → 确认 → 逐行创建
```

### 反思

1. **SSE 事件类型遗漏导致功能静默失败**：最初实现 Excel AI 流式解析时，新增了 `ExcelResult` SseEvent 变体，但前端 SSE client 的 switch-case 只匹配了 `result`，没有 `excel_result`。`ExcelResult` 事件被静默丢弃，`onResult` 回调从未触发。这次统一到 `Result` 变体后，前端无需修改即可正确处理。教训：**新增任何事件类型/枚举变体时，必须全链路检查所有消费者的匹配逻辑**。

2. **两套 prompt 维护成本高**：`build_excel_prompt()` 和 `build_system_prompt()` 有大量重复内容（分类描述、标签描述、属性描述、品牌识别规则），修改一处时另一处容易遗漏。统一到 `build_system_prompt()` + 增加表格处理 section 后，维护成本降低。教训：**当发现两个 prompt/模板有超过 50% 内容重复时，应该合并而非继续维护两个副本**。

## 2026-05-14 展示规则独立页面实施复盘

### 实施内容

1. **新建 `/views` 路由**：独立的规则视图页面，左右双栏布局
   - 左栏：规则列表（垂直菜单式）+ "管理规则 →" 链接
   - 右栏：按选中规则展示物品（list 模式复用 ItemListTable，summary 模式展示汇总卡片）
   - 空状态：未选中规则时显示"请选择一个展示规则"
   - 纯浏览模式：物品列表不响应点击，无详情面板，无 CRUD/批量操作
2. **导航新增"规则视图"入口**：在物品库和活动模板之间
3. **物品库页面精简**：移除 displayRules/selectedRuleId/ruleConfig 状态、applyRule() 函数、规则下拉框、summary 视图渲染

### 反思

1. **页面拆分时数据加载是独立的**：新页面 `/views` 和物品库页面各自独立调用 API 加载数据（items/categories/tags/attrDefs/displayRules）。两个页面之间没有共享状态，切换页面时会重新加载。这对于当前规模是合理的，但如果数据量很大，未来可考虑提取共享的数据层。

2. **ItemListTable 的 selectable 和 onSelect 配合**：在纯浏览模式下，`selectable={false}` 隐藏复选框，`onSelect={() => {}}` 传入空函数（因为 onSelect 是必填 prop）。不需要修改 ItemListTable 组件本身。

3. **`+page.ts` 不是必须的**：由于根 `+layout.ts` 已设置 `ssr = false` 和 `prerender = true`，views 路由无需单独的 `+page.ts`。只有需要覆盖这些设置的路由（如 `trips/[id]` 设置 `prerender = false`）才需要。

4. **summary 视图的搬运是纯复制**：summary 渲染模板和 CSS 从 items 页面完整搬运到 views 页面，无需修改逻辑。搬运后 items 页面删除了对应的渲染分支和 CSS。

## 2026-05-15 AI 整理 — 品类特有属性补充复盘

### 实施内容

1. **新建 migration `009_category_attrs.sql`**：新增 `food_type`（食品类型）、`electronics_type`（电子类型）、`body_parts_secondary`（副覆盖）属性定义，更新 `body_parts` 的 options 为 CDDA 风格（头/眼/口/颈/躯干/手臂/手/腿/脚/全身）
2. **`build_organize_prompt()` 增强**：签名新增 `attr_defs: &[AttributeDefinition]`，prompt 插入属性定义列表，物品描述追加非基础 attrs 值，新增第 6 项检查（品类特有属性缺失）
3. **`organize_preview()` handler**：从 DB 加载 `attr_defs` 传入 `build_organize_prompt()`
4. **`AiOrganizeModal` 动态 attrs diff**：新增 `attrDefs` prop，在 update diff 列表中动态渲染非基础属性的键值变化（通过 `attrDefs.find()` 查找中文 label）
5. **`items/+page.svelte`**：将 `attrDefs` 传递给 `<AiOrganizeModal>`

### 反思

1. **`AttributeDefinition.category_scope` 是 `String` 非 `Option<String>`**：最初写了 `.as_deref().unwrap_or("全局")`，编译器报错"no method `as_deref` on `String`"。空字符串表示全局，用 `.is_empty()` 判断即可。教训：**修改模型字段前先确认其确切类型，尤其是 `String` vs `Option<String>` 的区别**。

2. **`build_organize_prompt()` 签名变更后编译器自动找到所有调用方**：Rust 的类型检查确保了 `organize_preview()` 必须更新参数。如果是 Python/JS 这样的动态语言，遗漏调用方会是运行时错误。教训：**静态类型语言的好处——函数签名变更后编译器帮你找到所有遗漏的调用点**。

3. **migration 模式选择**：`food_type`/`electronics_type`/`body_parts_secondary` 用 `INSERT OR IGNORE`（幂等），`body_parts` 用 `UPDATE`（因为 `001_initial.sql` 已 seed，需要覆盖 options）。如果 `body_parts` 行不存在（极不可能），UPDATE 是静默 no-op，不会出问题。

4. **前端 attrs diff 中 `attrDefs.find()` 是 O(n²)**：每个 attrs key 都遍历 `attrDefs` 查找 label。当前 attrDefs 数量 < 20，每个 action 的 attrs 变化通常 < 5 个 key，性能可忽略。但如果 attrDefs 数量增长到数百，应考虑预建 Map。教训：**O(n²) 在数据量小时不是问题，但要意识到并在数据增长时重构**。

5. **物品描述中"非基础属性"的定义**：basic_keys 定义为 `["name", "brand", "model", "notes", "default_qty"]`，这些已在前面的固定格式中展示。其余 attrs（如 body_parts/food_type/electronics_type/waterproof/season 等）作为额外信息追加，帮助 LLM 判断是否需要补充。这个定义需要与 prompt 中第 6 项检查的 key 名保持一致。

## 2026-05-15 AI 整理误删标签修复复盘

### Bug: AI 整理建议删除所有标签

**现象**：加了规则 8（不要删除已有的正确标签）后，AI 仍然建议删除所有物品的标签。

**根因**：规则 2（字段错位）的后两个要点存在歧义：
```
- tag（标签）和名称之间的信息分配不合理
- 名称中包含了本应作为标签的信息，或反之
```
"或反之"告诉 LLM：名称已含标签信息 → 标签是"名称中本应作为标签的信息"的反面 → 标签多余应删除。这两个要点与规则 3（缺少标签→加标签）的正确方向冲突，且给了 LLM 删除标签的理由。

**修复**：规则 2 缩减为只保留 model 错位一个要点，删除两个 tag 相关要点。正确的"加标签"方向由规则 3 覆盖，"不删标签"由规则 8 兜底。

### 教训

1. **Prompt 中的"或反之"是危险表述**：给 LLM 一个正确的方向和一个反向的可能性，LLM 可能选择误解的方向。教训：**prompt 中只描述正确的操作方向，不要提供反向可能性——给 LLM 两个选择，它可能选错的**。

2. **多个规则覆盖同一主题会产生冲突**：规则 2（字段错位→tag 可能多余）和规则 3（缺少标签→加标签）从不同角度涉及 tag，规则 8（不要删除标签）是后来的防御补丁。三个规则互相角力，LLM 不知道哪个优先级更高。教训：**当一个主题被多个规则交叉覆盖时，应该合并为一条明确的规则，而非用多条规则互相补充/制衡**。

### 第三轮：从模板结构中移除 tag_name（最终根治）

第二轮修改后 AI 仍删标签。根因更深层：**模板结构 > 文字规则**。只要 `tag_name` 字段出现在 update 模板中，AI 就认为它是每次 update 都可以操作的字段。AI 进入"修改此物品"模式后，看到模板中所有可用字段，倾向于"顺便都修好"。

**修复**：从 update 模板的 `fields` 中完全移除 `tag_name` 字段。标签操作降级为规则 3 中描述的"特殊情况"（仅在物品缺少标签时补充），AI 不会在每次 update 时想到它。同时 split 模板中的 `"标签或null"` 改为 `"标签名"`，消除 null 是合法选项的暗示。

### 终极教训

| 轮次 | 做了什么 | 为什么不够 |
|------|---------|-----------|
| 1 | 删除规则 2 歧义要点 | 模板还在教"设为 null 表示清除标签"——主动教学 > 被动省略 |
| 2 | 改模板注释 + 正面规则 8/9 | `tag_name` 字段还在模板里——AI 看到可用字段就想用，文字禁止指令不够强 |
| 3 | **从模板删除 tag_name** | 标签操作降级为规则中描述的"特殊情况"，AI 不会在每次 update 时想到它 |

**Prompt 工程核心原则：结构（模板/字段）比文字（规则/注释）强一个数量级。要禁止某种行为，最有效的方式是从结构中移除触发该行为的入口，而不是在文字中说"不要做 X"。**"

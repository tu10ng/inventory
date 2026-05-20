# Inventory — 出行物品清单管理系统

## *** 必须使用中文思考 ***

## *** 项目的关键技术决策要提供选项, 让用户选择 ***

## *** 每次修改问题后, 都要增加一个"反思"的步骤, 反思为什么之前会做错, 并且一定要把复盘原因写入CLAUDE.md, 避免下次重犯 ***

## *** 测试时不要删除我的真实数据库, 要测试请写测试用例 ***

## *** 严禁执行 `rm -f inventory.db*` 或任何删除/清空 inventory.db 的操作！数据库是用户真实数据，只能用 cargo test （内存数据库）验证 migration 正确性 ***

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
│       └── 002_tag_scope.sql  type_scope 列迁移
├── frontend/             SvelteKit 2 + Svelte 5 (runes) + TypeScript
│   └── src/
│       ├── app.css               全局样式 + 物品栏暗色主题 + 打印样式
│       ├── lib/api/client.ts     fetch 封装，base path /api
│       ├── lib/types/index.ts    TS 类型定义
│       ├── lib/utils/status.ts   状态类型/选项常量（动态 API 加载）
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
| types | 物品子类型类型，每个 type 归属一个 category |
| items | 物品库（name, brand, model, category_id, default_qty, notes, type_id, attrs） |
| attribute_definitions | 物品属性定义（key/label/attr_type/config/category_scope/type_scope） |
| status_definitions | 状态定义（scope: item/trip） |
| activities | 活动模板（如"徒步"） |
| activity_slots | 活动槽位（slot_name, category_id, types, default_item_id, is_essential） |
| activity_slot_tags | 槽位↔类型多对多关联 |
| tips | 活动小贴士 |
| people | 人员 |
| trips | 行程（name, activity_id, start/end_date, status） |
| trip_items | 行程物品（qty, checked, item_status, notes, person_id, is_essential, slot_id） |
| llm_configs | LLM 模型配置（task/provider_name/base_url/api_key/model/is_active） |
| _migrations | 迁移追踪表 |

trip status: `planning` → `packing` → `done`

item_status 动态定义于 `status_definitions` 表（scope: item）

## API 路径约定

RESTful，前缀 `/api`。

| 方法 | 路径 | 说明 |
|------|------|------|
| GET/POST | `/api/categories` | 分类列表/创建 |
| PUT/DELETE | `/api/categories/{id}` | 更新/删除分类 |
| GET/POST | `/api/types` | 类型列表/创建 |
| PUT/DELETE | `/api/types/{id}` | 更新/删除类型 |
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
| GET | `/api/llm-configs` | LLM 配置列表（api_key 脱敏） |
| PUT | `/api/llm-configs/{id}` | 更新 LLM 配置 |
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
- `SseEvent` enum（`#[serde(type = "type")]`）区分 thinking/progress/result/error
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

1. **`type_scope` 列遗漏**：`002_tag_scope.sql` 迁移加了列，但只更新了 `attributes.rs` 的 CRUD handler，export/import 是后来加的功能，被遗漏。教训：**Schema 变更时应该全局 grep 所有引用该表的 SQL**（`grep "attribute_definitions"` 确认所有引用点）。

2. **`organize_apply` 的 `new_tags` 永远为空**：`OrganizeApplyResponse` 从 `AiParseResponse`（有 `new_tags`）照搬，但 apply 阶段 type 已在 preview 阶段创建完毕。教训：**复制代码时必须审视每个字段在目标场景下是否还有意义，照搬结构体 ≠ 照搬逻辑**。

### P1 死代码根因

- `svelte-dnd-action` 是 slot+type 系统开发时预装的依赖，最终用了原生 HTML5 drag，但没有清理。教训：**依赖不应该"预装"，应该在真正需要时才加，加了不用就要删**。

### P2 复杂度问题根因

- `compute_resync_diff` 的 N+1 查询是逐步堆叠补丁的典型后果：最初只处理"新增槽位"，后来追加删除逻辑（3 个独立 if 分支），每次只在循环体内加分支，没人退一步想"这条 item name 可以批量查"。教训：**当一个函数内部出现 3 个以上结构相似的代码块时，就该考虑提取公共逻辑**。

### P3 前端质量问题根因

- **"动态化改造不彻底"** 是最深层的问题：status 改为动态加载了，但 UI 颜色还在用硬编码 CSS；columns 动态加载了，但 `cellValue.ts` 只覆盖了 2/4 个核心字段。这类"改一半"的问题比不做更危险——给人"已支持动态化"的假象。教训：**做动态化改造时，必须全链路追踪：API → 类型 → 工具函数 → 组件渲染，每个环节都要确认不再有硬编码**。

### P4 文档腐烂根因

- 三阶段架构重构后（分类用户化→状态动态化→属性动态化），没有人系统地更新文档。大型重构的 checklist 必须包含"更新所有相关 CLAUDE.md"。

## 2026-05-12 统一属性系统实施复盘

### 实施中踩的坑

1. **table-rebuild 模式在 items 表上失败**：`rebuild_trip_items_fk` 和 `rebuild_trips_table` 都成功使用 CREATE→INSERT→DROP→RENAME 模式重建表，但 items 表有多个子表（trip_items、activity_slots）的 FK 引用。即使 `PRAGMA foreign_keys = OFF`，SQLite 仍拒绝 DROP TABLE items（错误信息不明确，只说 RENAME 时 items 已存在）。教训：**table-rebuild 模式在父表（被 FK 引用的表）上不可行，应改用 in-place UPDATE 迁移**。

2. **NOT NULL 约束的遗留列**：旧表 `name` 列有 NOT NULL 约束，新的 INSERT 只提供 `(category_id, type_id, attrs)` 不提供 `name`，导致 INSERT 失败。教训：**Schema 迁移不能只改代码的 SELECT/INSERT，还要检查所有列的约束条件。旧列不清除时，约束仍然生效**。

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

- **没有种子 type 数据**：`update_category_change` 测试使用了 `type_id: Some(1)`，但 migrations 只 seed 了 categories 和 status_definitions/attribute_definitions，没有 seed types。测试需要手动插入 type 或使用不存在的 type id。教训：**测试数据不能假设生产环境的种子数据完整，要显式准备或检查**。

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

**根因**：`handleFieldUpdate` 的原始逻辑只有两个分支：top-level 字段（`category_id`/`type_id`）直接写入，其他字段都视为 attrs 内的子字段，用 `{ ...attrs, [field]: value }` 包裹。但当 `ItemDetailPanel.updateAttr()` 调用时，`field` 就是 `'attrs'`，`value` 已经是完整的 attrs 对象（在 `updateAttr` 中已做过 `{ ...item.attrs, [key]: value }` 合并）。`handleFieldUpdate` 又做了一次 `{ ...attrs, attrs: value }`，导致后端收到 `{ attrs: { name: "旧", attrs: { name: "新" } } }`，旧值覆盖新值。

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

1. 后端 `POST /api/items/batch` 的 `update` action 扩展支持 `type_id`（含设 null）和 `attrs` merge（逐 item + transaction + legacy 列同步）
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
2. **Prompt 动态构建**：分类体系、类型列表、属性定义都从 DB 实时查询后嵌入 prompt，而非硬编码。
3. **两种导入路径共存**：AI 智能模式（适合列名不规范）和手动映射模式（适合列名已规范），用户可选择。
4. **人工可在 AI 预览阶段修正**：AI 解析结果以可编辑表格形式展示，支持 InlineEdit 修改 name/brand/model/default_qty，以及下拉选择 category/type。

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

2. **两套 prompt 维护成本高**：`build_excel_prompt()` 和 `build_system_prompt()` 有大量重复内容（分类描述、类型描述、属性描述、品牌识别规则），修改一处时另一处容易遗漏。统一到 `build_system_prompt()` + 增加表格处理 section 后，维护成本降低。教训：**当发现两个 prompt/模板有超过 50% 内容重复时，应该合并而非继续维护两个副本**。

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

1. **页面拆分时数据加载是独立的**：新页面 `/views` 和物品库页面各自独立调用 API 加载数据（items/categories/types/attrDefs/displayRules）。两个页面之间没有共享状态，切换页面时会重新加载。这对于当前规模是合理的，但如果数据量很大，未来可考虑提取共享的数据层。

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

## 2026-05-15 AI 整理误删类型修复复盘

### Bug: AI 整理建议删除所有类型

**现象**：加了规则 8（不要删除已有的正确类型）后，AI 仍然建议删除所有物品的类型。

**根因**：规则 2（字段错位）的后两个要点存在歧义：
```
- type（类型）和名称之间的信息分配不合理
- 名称中包含了本应作为类型的信息，或反之
```
"或反之"告诉 LLM：名称已含类型信息 → 类型是"名称中本应作为类型的信息"的反面 → 类型多余应删除。这两个要点与规则 3（缺少类型→加类型）的正确方向冲突，且给了 LLM 删除类型的理由。

**修复**：规则 2 缩减为只保留 model 错位一个要点，删除两个 type 相关要点。正确的"加类型"方向由规则 3 覆盖，"不删类型"由规则 8 兜底。

### 教训

1. **Prompt 中的"或反之"是危险表述**：给 LLM 一个正确的方向和一个反向的可能性，LLM 可能选择误解的方向。教训：**prompt 中只描述正确的操作方向，不要提供反向可能性——给 LLM 两个选择，它可能选错的**。

2. **多个规则覆盖同一主题会产生冲突**：规则 2（字段错位→type 可能多余）和规则 3（缺少类型→加类型）从不同角度涉及 type，规则 8（不要删除类型）是后来的防御补丁。三个规则互相角力，LLM 不知道哪个优先级更高。教训：**当一个主题被多个规则交叉覆盖时，应该合并为一条明确的规则，而非用多条规则互相补充/制衡**。

### 第三轮：从模板结构中移除 type_name（最终根治）

第二轮修改后 AI 仍删类型。根因更深层：**模板结构 > 文字规则**。只要 `type_name` 字段出现在 update 模板中，AI 就认为它是每次 update 都可以操作的字段。AI 进入"修改此物品"模式后，看到模板中所有可用字段，倾向于"顺便都修好"。

**修复**：从 update 模板的 `fields` 中完全移除 `type_name` 字段。类型操作降级为规则 3 中描述的"特殊情况"（仅在物品缺少类型时补充），AI 不会在每次 update 时想到它。同时 split 模板中的 `"类型或null"` 改为 `"类型名"`，消除 null 是合法选项的暗示。

### 终极教训

| 轮次 | 做了什么 | 为什么不够 |
|------|---------|-----------|
| 1 | 删除规则 2 歧义要点 | 模板还在教"设为 null 表示清除类型"——主动教学 > 被动省略 |
| 2 | 改模板注释 + 正面规则 8/9 | `type_name` 字段还在模板里——AI 看到可用字段就想用，文字禁止指令不够强 |
| 3 | **从模板删除 type_name** | 类型操作降级为规则中描述的"特殊情况"，AI 不会在每次 update 时想到它 |

**Prompt 工程核心原则：结构（模板/字段）比文字（规则/注释）强一个数量级。要禁止某种行为，最有效的方式是从结构中移除触发该行为的入口，而不是在文字中说"不要做 X"。**

## 2026-05-15 物品库属性优化复盘

### 实施内容

1. **Migration 010**: 修正 14 个属性定义的 `category_scope`（服装专属/服装+装备/电子专属），修复 AI 创建属性时 `"[]"` 误写为合法 scope 的 bug
2. **db.rs `clean_out_of_scope_attrs()`**: 移除每个物品 attrs 中不在 scope 内的属性 key，幂等（通过 `_migrations` 表 `011_clean_attrs_done` 标记）
3. **ai.rs**: 修复两处 `"[]"` bug（`extract_new_attr_defs_from_text` 构造和 SQL INSERT），prompt 中增加 scope 信息 + 规则 7（只填充物品对应分类的属性值）
4. **Migration 012**: 拆分"其他"品类——类型/物品/槽位迁移 + 品类重命名（洗漱→家居、证件→服务）

### 反思

1. **Migration 文件占位符与 `_migrations` 追踪的时序冲突**：最初计划创建 `011_clean_attrs.sql` 占位文件，让 `clean_out_of_scope_attrs()` 检查 `_migrations` 是否已有 011 记录来判断是否已执行。但 `run_all_setup()` 中 `run_migrations()` 先执行，会自动记录 011 文件到 `_migrations`，导致 `clean_out_of_scope_attrs()` 看到记录后跳过，清理永远不会执行。教训：**当 Rust 代码需要在 migration 之后执行副作用，且需要幂等标记时，不要用同名的 migration 文件做标记——用自定义标记名（如 `011_clean_attrs_done`）或独立追踪表**。

2. **`unwrap_or_else` 闭包返回类型必须匹配**：`tracing::warn!()` 返回 `()`，但 `unwrap_or_else` 期望闭包返回 `SqliteQueryResult`。改用 `if let Err(e) = ... { tracing::warn!(...) }` 模式避免类型不匹配。

3. **前端零改动**：因为 `attrMatchesScope()` 已正确实现逗号分隔 scope 解析（`split(',').filter(Boolean).map(Number)`），且 `ItemForm`/`ItemDetailPanel` 已通过它过滤属性。数据库 scope 数据修正后前端自动生效。教训：**好的抽象层能让数据层的修正对 UI 层透明**。

## 2026-05-16 enrich 联网搜索+LLM 属性补全复盘

### 实施内容

1. **db-manage SKILL.md**：新增命令 7 `enrich`，allowed-tools 增加 `WebSearch`
2. **首次运行 `--limit 5`**：扫描 61 个物品的 292 个缺失属性，选取 5 个物品进行搜索→提取→预览→写入

### 流程验证

- 5 个物品共 36 个缺失属性，高置信度 29 个已写入，剩余 7 个确属语义不匹配（body_parts 对水壶/驱蚊液无意义）
- VAN RYSEL Racer 骑行服和 RCR PRO 背带裤两个物品属性已全部补全（14/14）
- 搜索策略有效：迪卡侬子品牌补"迪卡侬"前缀提高召回率，专业评测站（road.cc/biketo.com）有详细规格

### 反思

1. **属性 scope 语义不精确**：`body_parts`/`body_parts_secondary`/`waterproof` 的 `category_scope` 设为 `1,2`（服装+装备），但水壶和驱蚊液是装备，这些属性对它们无意义。scope 只能表达"品类级别"的适用性，无法表达"品类内子类型的适用性"。教训：**当属性在品类内只有部分物品适用时，用 scope 过滤会误伤。应接受这类属性可能为空的合理性，而非强制补全**。

2. **WebSearch 的搜索词设计影响结果质量**：迪卡侬子品牌名（QUECHUA/FORCLAZ/VAN RYSEL 等）搜索结果较少，但补"迪卡侬"后召回率显著提高。教训：**品牌别名/从属关系需要在搜索词中显式补充，不能假设搜索引擎知道子品牌=母品牌**。

3. **enrich 的预览→确认模式是正确的设计**：计划中的 preview → confirm 模式让用户可以在写入前看到每个推断值的来源和置信度。这与之前 resync preview 的教训一致——**批量修改操作必须先展示将发生的变化**。

4. **属性写入后未同步 legacy 列**：当前 enrich 只更新 `attrs` JSON 列，未像 Rust handler 那样同步更新 legacy 列（name/brand/model）。但当前 items 表的 legacy 列已被 item attrs 系统替代，所以这不是问题。但如果未来有其他系统直接读 legacy 列，就会不一致。教训：**JSON 列更新时，如果存在对应的 shadow legacy 列，确认是否需要同步**。

## 2026-05-17 标签→类型 全面改名 + 树形层级架构复盘

### 实施内容

全项目范围的重命名（DB/API/前后端代码/文档/测试）+ 类型表新增 `parent_id` 树形层级 + 新增 `/types` 树形浏览页面。

### 验证结果

- `cargo check` 0 错误
- `cargo test` 53 passed
- `pnpm check` 0 错误（仅预存 a11y 警告）
- `pnpm test` 49 passed

### 反思

1. **大规模改名的效率策略**：20+ 文件的跨栈改名不应逐个 Edit 手改。本次采用的模式：**先用 sed 做机械性批量替换（tag_id→type_id、Tag→Type、FROM tags→FROM types 等），再用 Edit 做结构性增改（新增 parent_id、tree handler、types 页面等），最后用静态检查（cargo check / pnpm check）捕获遗漏**。这个"sed 扫射 → 精确修补 → 编译器兜底"的三阶段模式最小化了遗漏风险。

2. **sed 替换的边界问题**：`s/\btag\b/type/g` 会错误替换 `pill-tag` 这类视觉样式 CSS 类名、`added-tag` 等与 DB 概念无关的标识符。教训：**跨文件 sed 批量替换后，必须用 `grep -rn '\bold_word\b'` 残留检查来判断是否需要人工审查**。本次保留了两个视觉样式类（`.pill-tag`、`.added-tag`）不替换。

3. **sed 无法处理的 Rust 语法**：`Type { ... }` 结构体字面量缺少新增字段 `parent_id` 时，sed 无法自动补全。教训：**新增结构体字段后，用 `grep 'StructName {' src/` 全局搜索所有构造点逐一补全**，本次 activities.rs 和 ai.rs 的测试中有 3 处遗漏。

4. **前端测试的 `parent_id` 遗漏**：Type fixture 在 sed 后缺少 `parent_id`，`pnpm check` 报了类型错误才被发现。教训：**接口新增必填字段后，用 `grep 'InterfaceName'` 全局搜索所有 mock/fixture 构造点**。

5. **`$derived(() => {})` 的 TypeScript 类型问题**：在 Svelte 5 runes 中，`let x = $derived(() => { ... })` 被 TypeScript 推断为返回函数类型而非值类型，需要用 `$derived.by(() => { ... })` 或 `$derived(expression)`。本次 types 页面的 `filteredItems` 和 `breadcrumb` 遇到了这个问题。

6. **Svelte 5 `{#snippet}` 不能当组件用**：在 `{#each}` 块中调用递归 snippet 时，不能用 `<SnippetName prop={val} />` 的组件语法，必须用 `{@render SnippetName({ prop: val })}`。这是 Svelte 5 snippet 与 Svelte 4 组件调用的关键差异。

7. **itemFilters 递归匹配缺少 `return true`**：为类型筛选加了 `getDescendantTypeIds` 递归匹配逻辑后，if 块末尾缺少显式 `return true`，导致匹配成功时隐式返回 `undefined`（falsy），所有物品被误筛掉。教训：**在回调函数中用 if-else 链做多分支判断时，每个分支末尾必须有显式 return，不能依赖 fallthrough**。

8. **前后端同改的并行策略**：本次使用两个 Agent 并行修改前后端，后端约 2 分钟完成，前端约 26 分钟。前后端的改名互不依赖（只需约定好新命名），可以完全并行。但 Agent 的逐文件 Edit 模式对大规模改名效率低——后端用 sed 只花了十几秒。教训：**大规模机械改名优先用 sed 批量处理，Agent 只处理需要理解上下文的结构性改动**。

## 2026-05-17 物品库类型树形多级分组实施复盘

### 实施内容

物品库按类型分组从扁平改为树形多级嵌套（利用 `parent_id` 字段），UI 从虚线 fieldset 改为可折叠块状。

### 涉及文件

| 文件 | 改动 |
|------|------|
| `itemFilters.ts` | 新增 `TypeTreeGroup` 接口 + `groupItemsByTypeTree()` |
| `ItemGroupBlock.svelte` | fieldset→可折叠块，`children`/`depth` props，递归 self-import |
| `ItemListTable.svelte` | `groupedData` 类型扩展 `tree?` 字段，树形渲染分支 |
| `items/+page.svelte` | `groupByKey === 'type'` 时调用 `groupItemsByTypeTree` |

### 反思

1. **`<svelte:self>` 在 Svelte 5 已废弃**：Svelte 5 要求改用 self-import（`import ItemGroupBlock from './ItemGroupBlock.svelte'`），然后用 `<ItemGroupBlock>` 递归调用自身。**教训：使用 Svelte 5 时应关注 deprecated API 变化，`<svelte:self>` → self-import 是 Svelte 5 的明确废弃**。

2. **`groupedData` 类型扩展用了可选 `tree` 字段向后兼容**：`tree?: TypeTreeGroup[]` 只在 `groupByKey === 'type'` 时有值，其他分组模式不变。ItemListTable 用 `{#if groupBy.key === 'type' && catData.tree}` 做条件分支，`{:else}` 保持原有扁平路径。这种"可选字段 + 运行时判断"的模式比 Union 类型更简单，不需要修改所有调用方。

3. **树形分组函数需处理 3 种 edge case**：正常 `parent_id` 链、父类型在另一品类（视为根）、不在树中的孤立类型有物品（作为独立根节点追加）。这些情况在数据不规范时会出现，不能假设 `parent_id` 链完整。

4. **`groupItems` 被调用了两次**：初始实现中非 type 分组时 `groupItems` 被调用了两次——一次取 `.groups`、一次取 `.ungrouped`。已修正为解构赋值一次调用。教训：**解构赋值可以避免重复计算，尤其是在 `$derived.by` 中频繁执行的代码**。

## 2026-05-19 删除 Categories 用类型树根节点替代复盘

### 实施内容

全栈删除了 `categories` 表和 `category_id` 列（后端 models/handlers + 前端 types/utils/components/pages），用类型树根节点（types WHERE parent_id IS NULL）替代分类概念。

### 验证结果

- `cargo check` 0 错误
- `cargo test` 49 passed
- `pnpm check` 0 错误（仅预存 a11y 警告）
- `pnpm test` 48 passed
- 删旧 DB 后 `cargo run` 启动成功，迁移无错

### 反思

1. **`execute_sql_file` 的 `split(';')` 与 `PRAGMA foreign_keys` 的致命组合**：`execute_sql_file` 按 `;` 切分 SQL 后逐条执行，每条语句可能从连接池获取不同连接。`PRAGMA foreign_keys = OFF` 是连接级设置，在语句 A 的连接上设置后，语句 B 可能获取另一个连接（FK 仍 ON），导致 `DROP TABLE types` 时 FK 约束失败。教训：**任何需要 PRAGMA 跨语句生效的 migration，必须用 Rust 代码在单个连接（`pool.acquire()`）上执行全部语句，不能用 `split(';')` 文件模式**。

2. **`split(';')` 会在注释中的 `;` 处分片**：迁移文件注释中写了 `split(';')` 字样，文本中的 `;` 被 `split(';')` 切分，导致注释片段（如 `')-based migration execution...`）被当作 SQL 执行，触发语法错误。教训：**migration 文件注释中不要出现 `;` 字符，因为 `split(';')` 不区分注释和 SQL**。

3. **两次 Agent 并行策略**：前后端 Agent 完全独立运行（各约 45 分钟），后端 agent 先完成（159 tool uses），前端随后（113 tool uses）。两个 Agent 修改了 36+ 个文件，互不冲突。但后端 Agent 在处理 `execute_sql_file` 的 FK 问题时浪费了很多轮次（反复尝试在 SQL migration 中修表结构），最终还是由主 session 接手改为 Rust 单连接模式解决。教训：**Agent 对基础设施限制（如连接池语义）的理解不如人类，这类底层问题应由主 session 处理**。

4. **migration 的 Rust 代码化是正确方向**：复杂的 migration（涉及 FK 操作、表重建）不应该用 `split(';')` 的 SQL 文件，而应该在 `db.rs` 中用 Rust 函数 + 单连接执行。SQL 文件只适合简单的 DDL（CREATE TABLE、ALTER TABLE ADD COLUMN、INSERT OR IGNORE 种子数据）。教训：**Migration 文件的内容复杂度应有上限——涉及 PRAGMA、FK 重建、多表联动的操作，一律用 Rust 函数**。

5. **数据映射顺序至关重要**：`migrate_remove_categories()` 中，必须先 remap `parent_id` 和 `category_scope`（此时 types 表中 category_id 仍保留旧值），再 zero 掉 category_id。如果顺序反了，映射信息就丢失了。教训：**涉及数据映射的 migration，必须明确区分"捕获阶段"和"修改阶段"，确保映射所需的旧值在修改前已读取**。

6. **🚨 生产数据丢失事故**：为了测试 clean migration，执行了 `rm -f inventory.db*` 删除了用户的真实数据库。数据库包含用户手动添加的全部物品，没有任何备份。教训：**CLAUDE.md 虽然写了"不要删除数据库"，但措辞不够强硬。实际操作中，为了验证 migration，"删除 DB → 重启 → 检查" 是常见的测试冲动。必须用以下方式替代：（1）`cargo test` 验证——测试在 `sqlite::memory:` 上运行所有 migration，完全验证逻辑正确性；（2）`dev.sh` 启动后检查——先确认能正常启动，再用浏览器查看数据完整性。永远不要对 `inventory.db` 执行 `rm`。**

## 2026-05-19 OCR 导入增加 AI 多模态视觉识别复盘

### 实施内容

后端新增 `POST /api/ai/ocr-vision` 端点 + 前端 OrderImportModal 增加"AI 视觉识别"按钮。

### 反思

1. **Rust `\` 字符串续行与中文引号 `"` 的冲突**：`user_prompt` 使用 `"\` 续行模式包含中文文本 `"【官方正品】""限时特价"`，Rust lexer 将首个 `"` 解释为关闭字符串的引号，导致后面的 `【` 被当作 Rust token 报错。教训：**在 Rust 续行字符串中不能出现半角 `"`（即使在中文语境中表示引用）。替代方案：（1）使用 `concat!()` 宏逐行拼接，每行是独立字符串字面量；（2）将中文引号改用全角「」替代半角 ""**。

## 2026-05-20 修复 AI 智能添加 SSE 流式输出卡住不结束

### Bug 现象

AI 智能添加时，流式输出中的思考内容和 result 结果已经返回完了，但前端仍显示"AI 正在分析"，不切换到预览界面。

### 根因分析

**Bug A（主要原因）— `client.ts` SSE 读取循环的缓冲区竞态**：

```typescript
// 旧代码
while (true) {
    const { done, value } = await reader.read();
    if (done) break;  // ← 直接跳出，不处理 value！
    buffer += decoder.decode(value, { stream: true });
    // ...
}
```

当 TCP 层将最后的 `data: {"type":"result",...}\n` 和流结束信号（FIN）合并在同一数据包时，`reader.read()` 返回 `{ done: true, value: Uint8Array }`。代码在解码 `value` 之前就 `break` 了，`onResult` 永远不会触发，UI 永远卡在 `loading` 状态。

**Bug B（次要因素）— `KeepAlive::default()` 延长等待**：`drop(tx)` 后 stream 结束，但 KeepAlive 定时器让连接额外存活，拉长了卡住时间。

### 修复

1. **`client.ts`**：改为先处理 `value` 中的数据，再判断 `done`；`done` 时 flush buffer 中剩余内容
2. **`ai.rs`**：移除 `Sse::new(stream).keep_alive(KeepAlive::default())` → `Sse::new(stream)`
3. **流式 prompt 简化**：思考要求 2-5 → 1-3 句话，JSON 格式移除 `new_attrs` 字段（减少 AI 输出复杂度）
4. **后端自动提取 attrs**：新增 `extract_new_attr_defs_from_items()` 函数，从解析出的 item attrs 中自动检测未知 key 并创建属性定义，不再依赖 AI 输出 `new_attrs`

### 反思

1. **`done` 和 `value` 可能同时有值**：`ReadableStream.read()` 的 `{ done, value }` 不是互斥的——TCP 层的 FIN 可以与最后一个数据段合并在同一帧中。先判 `done` 后处理 `value` 会导致最后一帧数据被丢弃。教训：**读取流的 while 循环中，必须先处理 `value`（如果存在），再判断 `done` 是否退出。这个顺序对 ReadableStream、TCP socket、文件 I/O 等所有流式 API 都适用**。

2. **KeepAlive 与 SSE 的语义冲突**：KeepAlive 用于保持长连接（如心跳、等待新事件），但 `parse_items_stream` 的通道在 `drop(tx)` 后明确结束——所有数据已发送完毕，不需要保持连接。KeepAlive 在此场景下只会延迟客户端感知流结束。教训：**KeepAlive 适合"持续推送"的场景（如实时通知），不适合"一次性流式响应"的场景。要根据业务语义选择是否使用**。

3. **Prompt 复杂度直接影响流式解析成功率**：`new_attrs` 要求 AI 在流式模式下同时输出 items 和新属性定义，增加了 JSON 结构的复杂度，也增加了 `---JSON---` 分隔符后被截断的风险。移除 `new_attrs` 后 JSON 更简单，解析失败率降低。属性定义的创建交给后端代码兜底（从 attrs 自动提取），比依赖 AI 更可靠。教训：**AI 的输出格式应尽量简单，复杂的推导逻辑应放在代码中而非 prompt 中。代码兜底 > AI 承诺**。

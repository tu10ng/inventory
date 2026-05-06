# Routes — 页面路由

SvelteKit file-based routing，SPA 模式（`ssr = false`, `adapter-static`）。

## 路由清单

| 路径 | 文件 | 说明 |
|------|------|------|
| `/` | `+page.svelte` | 首页：最近行程列表 + 快速创建入口 |
| `/trips` | `trips/+page.svelte` | 行程列表：创建/删除/克隆行程 |
| `/trips/[id]` | `trips/[id]/+page.svelte` | 行程详情：双栏布局（ChecklistPanel + InventoryPanel） |
| `/items` | `items/+page.svelte` | 物品库：CDDA 风格双面板（分类表格列表 + 详情面板），列配置可自定义 |
| `/activities` | `activities/+page.svelte` | 活动模板：CRUD + 关联物品(含 is_essential) + 提示管理 |

## UI 设计原则

### 双面板 Master-Detail 模式
页面采用左右双面板：左面板是自洽的浏览器（工具栏 + 列表），右面板是被动响应的详情区。用户在左面板内完成"筛选 → 浏览 → 选中"的完整流程，右面板根据选中状态联动展示。

### 控件归属原则
控制某个面板内容的 UI 控件（搜索、筛选、列配置等）必须在该面板内部，不能脱离成页面级控件。工具栏横跨全宽会造成视觉割裂——看起来像页面级控件，但实际只影响一侧，破坏用户认知一致性。

### 面板内聚性
每个面板应是一个完整的交互单元：控件 → 内容 → 反馈都在同一个视觉容器内闭合。面板之间通过最小化的状态（如 selectedId）联动，而非共享控件。

## 布局

### +layout.svelte
- 左侧固定侧边栏（导航菜单）
- 右侧 `.content` 容器（`max-width: 1400px`），容纳双面板布局

### +layout.ts
```typescript
export const prerender = true;
export const ssr = false;
```

## 页面架构

### 首页 (`+page.svelte`)
简洁入口，展示最近行程和快速创建表单。

### 行程列表 (`trips/+page.svelte`)
- 创建行程表单（名称 + 活动模板 + 日期）
- 行程卡片列表（含状态 badge、克隆、删除按钮）

### 行程详情 (`trips/[id]/+page.svelte`)
核心页面，使用 SplitPane 双栏布局：
- **头部**：行程名称 + 日期 + 状态切换 + 克隆按钮
- **左栏 ChecklistPanel**：进度条 → 提示 → 操作栏（模板填充/同步预览确认/添加查重/批量/打印）→ 分类分组清单
- **右栏 InventoryPanel**：暗色物品库面板，搜索/筛选 + 卡片网格，点击添加到清单
- **右栏可折叠**：桌面端有折叠按钮，收起后左栏占满

`+page.ts`: `export const prerender = false`（动态路由）

### 物品库 (`items/+page.svelte`)
CDDA 风格双面板 Master-Detail 布局（遵循控件归属原则）：
- **左面板**：自洽的浏览器单元
  - 内置工具栏：SearchFilter + ColumnPicker（齿轮按钮，持久化 localStorage）+ 添加按钮
  - ItemListTable：CSS Grid 表格，按 category 分组可折叠，点击行选中物品
- **右面板 PanelContainer**：被动响应详情区，sticky 容器，条件渲染 ItemDetailPanel / ItemForm / 空状态
- 面板间通过 `selectedItemId` 最小状态联动
- 使用统计（从 `/api/item-stats` 加载，失败不阻塞页面）
- CRUD 流程：panelMode 状态（detail / edit / create / null）控制右面板内容

### 活动模板 (`activities/+page.svelte`)
- 左右布局：活动列表 | 选中活动的详情
- 详情包含：物品列表（可添加/移除，含 is_essential 星标切换）+ 提示列表

## 数据加载模式

所有页面使用相同模式：
```svelte
let data = $state<Type[]>([]);
async function load() { data = await api.get<Type[]>('/path'); }
$effect(() => { load(); });
```

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

## 布局

### +layout.svelte
- 左侧固定侧边栏（导航菜单）
- 右侧 `.content` 容器（`max-width: 1400px`）

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
CDDA 风格双面板布局：
- **工具栏**：SearchFilter + ColumnPicker（齿轮按钮，列选择持久化到 localStorage）+ 添加按钮
- **左面板 ItemListTable**：CSS Grid 表格，物品按 category 分组，分类标题可折叠，点击行选中物品
- **右面板 PanelContainer**：sticky 容器，条件渲染 ItemDetailPanel / ItemForm / 空状态
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

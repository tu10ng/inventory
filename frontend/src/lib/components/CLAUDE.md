# Components — 可复用 Svelte 组件

## 组件清单

### 布局组件

| 组件 | 职责 | 关键 props |
|------|------|-----------|
| `SplitPane.svelte` | 响应式双栏容器 | `left: Snippet`, `right: Snippet` |
| `CategoryGroup.svelte` | 可折叠分类组 | `icon, name, checked, total, collapsed, onToggle, children: Snippet` |

### 面板组件

| 组件 | 职责 | 关键 props |
|------|------|-----------|
| `ChecklistPanel.svelte` | 左栏：进度条 + 提示 + 分类清单 + 添加/批量操作 | `trip, tripItems(bindable), allItems, categories, tips, people, onPopulate, onResync` |
| `InventoryPanel.svelte` | 右栏：暗色物品库，搜索/分类筛选 + 卡片网格 | `items, categories, tripItemIds, onAddItem` |
| `PanelContainer.svelte` | 右面板 sticky 容器（Snippet children） | `children: Snippet` |
| `ItemDetailPanel.svelte` | 物品详情面板：属性/统计/标签展示 | `item, categories, tags, usageCount, onEdit, onDelete` |
| `ItemForm.svelte` | 物品创建/编辑表单 | `item?, categories, tags, onSave, onCancel` |

### 列表组件

| 组件 | 职责 | 关键 props |
|------|------|-----------|
| `ItemListTable.svelte` | 物品库表格：按分类分组 + 可折叠 + 多列渲染（text/number/bool/bar/stars/tag/weight） | `items, categories, tags, usageStats, visibleColumns, selectedItemId, collapsedCategories, onSelect, onToggleCategory` |
| `ColumnPicker.svelte` | 列选择下拉：齿轮按钮 + checkbox 列表，持久化到 localStorage | `visibleKeys(bindable)` |

### 行级组件

| 组件 | 职责 | 关键 props |
|------|------|-----------|
| `TripItemRow.svelte` | 清单单行：勾选 + 名称 + 内联编辑(qty/notes) + 状态/人员/删除 | `tripItem, itemInfo, people, selected, selectable, onToggle*, onUpdate*, onRemove` |
| `ItemCard.svelte` | 游戏风格物品卡片（行程详情 InventoryPanel 使用） | `name, brand, model, categoryIcon, qty, alreadyAdded, onclick` |

### 原子组件

| 组件 | 职责 | 关键 props |
|------|------|-----------|
| `ProgressBar.svelte` | 进度条（checked/total） | `checked, total` |
| `StatusBadge.svelte` | item_status 下拉选择 | `status, onchange` |
| `PersonBadge.svelte` | person_id 下拉选择 | `personId, people, onchange` |
| `InlineEdit.svelte` | 点击即编辑的文本/数字 | `value, type, oncommit, min, placeholder` |
| `SearchFilter.svelte` | 搜索框 + 分类下拉 | `search, categoryId, categories, onSearchChange, onCategoryChange, dark` |

## 依赖关系

```
SplitPane (行程详情)
├── ChecklistPanel
│   ├── ProgressBar
│   ├── CategoryGroup
│   └── TripItemRow
│       ├── StatusBadge
│       ├── PersonBadge
│       └── InlineEdit
└── InventoryPanel
    ├── SearchFilter
    └── ItemCard

物品库页面 (items/+page.svelte) — Master-Detail 模式
├── 左面板（自洽浏览器：工具栏 + 列表）
│   ├── SearchFilter
│   ├── ColumnPicker
│   └── ItemListTable
└── 右面板（被动响应详情区）
    └── PanelContainer
        ├── ItemDetailPanel
        └── ItemForm
```

## 组件约定

- 所有 props 用 `$props()` + 内联类型注解
- 回调命名：`onXxx`（`onchange`, `onToggleCheck`, `onRemove`）
- Snippet slot 用 Svelte 5 的 `children: Snippet` + `{@render children()}`
- `ChecklistPanel.tripItems` 是 `$bindable()`，父组件可双向绑定
- 样式 scoped，暗色主题组件使用 `var(--inventory-*)` 变量
- `ItemCard` 和 `InventoryPanel` 使用暗色主题（`--inventory-bg` 系列）

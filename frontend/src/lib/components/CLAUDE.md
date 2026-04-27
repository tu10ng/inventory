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

### 行级组件

| 组件 | 职责 | 关键 props |
|------|------|-----------|
| `TripItemRow.svelte` | 清单单行：勾选 + 名称 + 内联编辑(qty/notes) + 状态/人员/删除 | `tripItem, itemInfo, people, selected, selectable, onToggle*, onUpdate*, onRemove` |
| `ItemCard.svelte` | 游戏风格物品卡片 | `name, brand, model, categoryIcon, qty, alreadyAdded, onclick` |

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
SplitPane
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
```

## 组件约定

- 所有 props 用 `$props()` + 内联类型注解
- 回调命名：`onXxx`（`onchange`, `onToggleCheck`, `onRemove`）
- Snippet slot 用 Svelte 5 的 `children: Snippet` + `{@render children()}`
- `ChecklistPanel.tripItems` 是 `$bindable()`，父组件可双向绑定
- 样式 scoped，暗色主题组件使用 `var(--inventory-*)` 变量
- `ItemCard` 和 `InventoryPanel` 使用暗色主题（`--inventory-bg` 系列）

# Utils — 工具函数和常量

## 文件

### status.ts

状态相关的类型映射和选项列表。

| 导出 | 类型 | 用途 |
|------|------|------|
| `STATUS_LABELS` | `Record<ItemStatus, string>` | item_status → 中文类型映射 |
| `STATUS_OPTIONS` | `{ value, label }[]` | 状态下拉选项（含 '无'） |
| `TRIP_STATUS_LABELS` | `Record<string, string>` | trip status → 中文类型映射 |

### 状态值对照

| item_status | 中文 |
|-------------|------|
| `''` | 无 |
| `need_buy` | 需购买 |
| `need_find` | 需寻找 |
| `need_charge` | 需充电 |
| `need_fetch` | 需取回 |
| `need_give` | 需带给 |

| trip status | 中文 |
|-------------|------|
| `planning` | 计划中 |
| `packing` | 打包中 |
| `done` | 已完成 |

### columns.ts

物品库列表的列定义和持久化配置。

| 导出 | 类型 | 用途 |
|------|------|------|
| `ALL_COLUMNS` | `ItemColumnDef[]` | 所有可用列定义（name/type/brand/model/weight/warmth/encumbrance/waterproof/breathable/env_protection/durability/usage） |
| `loadVisibleColumns()` | `() => string[]` | 从 localStorage 读取可见列 key 列表，默认 `['name','type','brand','model','weight','warmth','waterproof']` |
| `saveVisibleColumns()` | `(keys: string[]) => void` | 持久化可见列到 localStorage（key: `inventory-visible-columns`） |

- `name` 列强制可见，不可关闭
- 每列有 `render` 类型：`text` / `number` / `bool` / `bar` / `stars` / `type` / `weight`
- 每列有 `getValue(item, ctx?)` 函数从 Item 提取值，ctx 可携带 types 和 usageStats

# Utils — 工具函数和常量

## 文件

### status.ts

状态相关的标签映射和选项列表。

| 导出 | 类型 | 用途 |
|------|------|------|
| `STATUS_LABELS` | `Record<ItemStatus, string>` | item_status → 中文标签映射 |
| `STATUS_OPTIONS` | `{ value, label }[]` | 状态下拉选项（含 '无'） |
| `TRIP_STATUS_LABELS` | `Record<string, string>` | trip status → 中文标签映射 |

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

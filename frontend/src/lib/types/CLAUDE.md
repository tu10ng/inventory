# Types — TypeScript 类型定义

## 文件

`index.ts` — 所有接口和类型定义，与后端 `models.rs` 一一对应。

## 核心模型

| 类型 | 对应后端 | 说明 |
|------|---------|------|
| `Category` | `Category` | 物品分类（id, name, icon, sort_order） |
| `Item` | `Item` | 物品（id, name, brand, model, category_id, default_qty, notes） |
| `Activity` | `Activity` | 活动模板 |
| `ActivityItem` | `ActivityItem` | 活动↔物品关联（含 is_essential） |
| `Tip` | `Tip` | 活动小贴士 |
| `Person` | `Person` | 人员（id, name） |
| `Trip` | `Trip` | 行程（status: 'planning' \| 'packing' \| 'done'） |
| `TripItem` | `TripItem` | 行程物品（含 is_essential, person_id） |

## 辅助类型

| 类型 | 用途 |
|------|------|
| `ItemStatus` | `'' \| 'need_buy' \| 'need_find' \| 'need_charge' \| 'need_fetch' \| 'need_give'` |
| `TripItemWithInfo` | TripItem 扩展，附带 item_info 和 category |
| `ItemUsageCount` | 物品使用统计（item_id, trip_count） |
| `ItemUsageStats` | 物品使用详情（item_id, trips: TripRef[]） |
| `TripRef` | 行程引用（id, name, status） |
| `BulkUpdateTripItems` | 批量更新请求体（ids, checked?, person_id?, item_status?） |
| `DndItem` | 拖拽物品数据结构 |

## 维护规则

- 新增后端模型时同步添加对应 TS 接口
- 字段名与后端 JSON 保持一致（snake_case）
- 可选字段用 `T | null`（与后端 `Option<T>` 对应）

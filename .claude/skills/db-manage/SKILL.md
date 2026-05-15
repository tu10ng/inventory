---
name: db-manage
description: >
  Inventory 项目 SQLite 数据库管理。用于检查数据库健康状态、数据质量、
  属性 scope 覆盖、迁移状态、执行清理操作。当用户提到数据库、db、SQLite、
  数据检查、迁移、migration、scope、清理、数据质量等关键词时触发。
user-invocable: true
disable-model-invocable: false
argument-hint: "[health|scope|migrations|quality|cleanup|query|enrich]"
allowed-tools:
  - "Bash(sqlite3 *)"
  - "Bash(cat *)"
  - "Bash(ls *)"
  - "Bash(wc *)"
  - "WebSearch"
tags: [database, sqlite, maintenance, inventory]
---

# Inventory 数据库管理

管理 `backend/inventory.db` SQLite 数据库，提供健康检查、数据质量诊断、迁移管理和清理操作。

## 重要规则

- **绝对不要删除数据库文件** (`inventory.db` / `inventory.db-wal` / `inventory.db-shm`)
- 所有写操作前先展示将要改变的数据预览
- 清理操作需用户确认后执行

## 数据库路径

```
backend/inventory.db          # 主数据库文件
backend/inventory.db-wal      # WAL 日志
backend/inventory.db-shm      # WAL 共享内存
backend/migrations/           # 迁移 SQL 文件
```

---

## 命令 1: 健康检查 (`health`)

查看数据库整体状态：表行数、迁移数、WAL 大小。

### 执行步骤

1. 运行以下查询获取各表行数：

```bash
sqlite3 backend/inventory.db "
SELECT 'categories' AS 表, COUNT(*) AS 行数 FROM categories UNION ALL
SELECT 'tags', COUNT(*) FROM tags UNION ALL
SELECT 'items', COUNT(*) FROM items UNION ALL
SELECT 'attribute_definitions', COUNT(*) FROM attribute_definitions UNION ALL
SELECT 'status_definitions', COUNT(*) FROM status_definitions UNION ALL
SELECT 'display_rules', COUNT(*) FROM display_rules UNION ALL
SELECT 'relation_types', COUNT(*) FROM relation_types UNION ALL
SELECT 'item_relations', COUNT(*) FROM item_relations UNION ALL
SELECT 'activities', COUNT(*) FROM activities UNION ALL
SELECT 'activity_slots', COUNT(*) FROM activity_slots UNION ALL
SELECT 'activity_slot_tags', COUNT(*) FROM activity_slot_tags UNION ALL
SELECT 'tips', COUNT(*) FROM tips UNION ALL
SELECT 'people', COUNT(*) FROM people UNION ALL
SELECT 'trips', COUNT(*) FROM trips UNION ALL
SELECT 'trip_items', COUNT(*) FROM trip_items UNION ALL
SELECT '_migrations', COUNT(*) FROM _migrations
ORDER BY 表;
"
```

2. 查看已应用的迁移：

```bash
sqlite3 backend/inventory.db "SELECT filename FROM _migrations ORDER BY filename;"
```

3. 检查 WAL 文件大小：

```bash
ls -lh backend/inventory.db*
```

### 输出格式

用表格展示各表行数，标注异常（如空表、行数异常多/少）。

---

## 命令 2: 属性 Scope 检查 (`scope`)

检查 `attribute_definitions` 的 `category_scope` 设置是否合理，以及物品 attrs 中是否有越界属性。

### 执行步骤

1. 属性 scope 概览：

```bash
sqlite3 backend/inventory.db "
SELECT
  CASE WHEN category_scope = '' OR category_scope IS NULL THEN '全局'
       ELSE category_scope END AS scope,
  COUNT(*) AS 数量,
  GROUP_CONCAT(key, ', ') AS 属性
FROM attribute_definitions
GROUP BY category_scope
ORDER BY scope;
"
```

2. 检查 `category_scope = '[]'` 的残留（AI 创建的 bug 数据）：

```bash
sqlite3 backend/inventory.db "
SELECT key, label, category_scope
FROM attribute_definitions
WHERE category_scope = '[]';
"
```
预期：0 行。如果有结果，说明 scope 修正迁移未覆盖或 AI 后续又创建了有 bug 的属性。

3. 检查物品有无越界属性（原理：scope 非空的属性，物品 category_id 必须在 scope 内）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, i.category_id, c.name AS cat_name,
       json_extract(i.attrs, '$.name') AS item_name
FROM items i
JOIN categories c ON c.id = i.category_id
JOIN attribute_definitions ad ON ad.category_scope != '' AND ad.category_scope != '[]'
WHERE json_extract(i.attrs, '$.' || ad.key) IS NOT NULL
  AND json_extract(i.attrs, '$.' || ad.key) != ''
  AND json_extract(i.attrs, '$.' || ad.key) != '0'
  AND ',' || ad.category_scope || ',' NOT LIKE '%,' || i.category_id || ',%'
GROUP BY i.id
ORDER BY i.category_id, i.id;
```

### 输出

- Scope 分布表
- `'[]'` 检查结果（有/无）
- 越界物品列表（如果有）：物品名、当前分类、越界属性名

---

## 命令 3: 数据质量检查 (`quality`)

全面检查数据一致性问题。

### 检查项

1. **孤儿 tag**（tag 的 category_id 指向不存在的分类）：

```bash
sqlite3 backend/inventory.db "
SELECT t.id, t.name, t.category_id
FROM tags t LEFT JOIN categories c ON c.id = t.category_id
WHERE c.id IS NULL;
"
```

2. **孤儿 item**（item 的 category_id/tag_id 指向不存在的记录）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS name,
       i.category_id, i.tag_id
FROM items i
LEFT JOIN categories c ON c.id = i.category_id
WHERE c.id IS NULL;
"
```

3. **物品重名检查**（同品牌+型号+名称相同的物品）：

```bash
sqlite3 backend/inventory.db "
SELECT
  json_extract(attrs, '$.name') AS name,
  json_extract(attrs, '$.brand') AS brand,
  json_extract(attrs, '$.model') AS model,
  COUNT(*) AS cnt,
  GROUP_CONCAT(id) AS ids
FROM items
WHERE json_extract(attrs, '$.name') != ''
GROUP BY json_extract(attrs, '$.name'), json_extract(attrs, '$.brand'), json_extract(attrs, '$.model')
HAVING cnt > 1
ORDER BY cnt DESC;
"
```

4. **分类分布**（各品类物品数量 + 空品类）：

```bash
sqlite3 backend/inventory.db "
SELECT c.id, c.name, c.icon, COUNT(i.id) AS items
FROM categories c
LEFT JOIN items i ON i.category_id = c.id
GROUP BY c.id
ORDER BY c.sort_order;
"
```

5. **无标签物品**：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS name, c.name AS cat_name
FROM items i
JOIN categories c ON c.id = i.category_id
WHERE i.tag_id IS NULL;
"
```

6. **标签分布**（每个 tag 有多少物品）：

```bash
sqlite3 backend/inventory.db "
SELECT t.name AS tag, c.name AS category, COUNT(i.id) AS items
FROM tags t
LEFT JOIN items i ON i.tag_id = t.id
JOIN categories c ON c.id = t.category_id
GROUP BY t.id
ORDER BY items DESC;
"
```

### 输出

逐项展示检查结果，标注问题数量和严重程度。

---

## 命令 4: 清理越界属性 (`cleanup`)

移除物品 attrs 中不属于该分类的属性值（如营养品有 `body_parts`、`warmth_rating`）。

### 前置条件

- migration 010 (`010_fix_category_scope.sql`) 必须已应用
- 先运行 `scope` 命令确认越界情况

### 执行步骤

1. 先预览将要清理的内容（dry-run）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, i.category_id, c.name AS cat_name,
       json_extract(i.attrs, '$.name') AS item_name,
       ad.key AS attr_key,
       json_extract(i.attrs, '$.' || ad.key) AS attr_value
FROM items i
JOIN categories c ON c.id = i.category_id
JOIN attribute_definitions ad ON ad.category_scope != '' AND ad.category_scope != '[]'
WHERE json_extract(i.attrs, '$.' || ad.key) IS NOT NULL
  AND json_extract(i.attrs, '$.' || ad.key) != ''
  AND json_extract(i.attrs, '$.' || ad.key) != '0'
  AND ',' || ad.category_scope || ',' NOT LIKE '%,' || i.category_id || ',%'
ORDER BY i.category_id, i.id;
"
```

2. 展示统计摘要：每个分类有多少物品、多少越界属性。
3. **必须询问用户确认**后再执行实际清理。
4. 执行清理：删除 `clean_out_of_scope_attrs_done` 标记并重启后端（触发 `clean_out_of_scope_attrs()` 函数），或直接运行 SQLite JSON 操作逐物品清理。

```bash
# 方案 A: 重启触发（推荐，使用已有的 Rust 清理函数）
sqlite3 backend/inventory.db "DELETE FROM _migrations WHERE filename = '011_clean_attrs_done';"
# 然后重启后端

# 方案 B: 手动清理（逐 key 移除）
# 对于每个越界的 (item_id, key) 对执行：
# UPDATE items SET attrs = json_remove(attrs, '$.key_name') WHERE id = item_id;
```

---

## 命令 5: 迁移状态 (`migrations`)

查看迁移历史和文件对应关系。

```bash
# 已应用的迁移
sqlite3 backend/inventory.db "SELECT id, filename, applied_at FROM _migrations ORDER BY id;"

# 迁移文件列表
ls -1 backend/migrations/*.sql
```

对比迁移文件列表和 `_migrations` 表，标注未应用的文件（如果有）。

---

## 命令 6: 自定义查询 (`query`)

执行自定义 SQL 查询。

$ARGUMENTS 中 `query` 之后的内容作为 SQL 执行，不包含 `query` prefix 则视为只读 SELECT。

```bash
sqlite3 backend/inventory.db "$ARGUMENTS"
```

示例：
- `/db-manage query SELECT * FROM categories`
- `/db-manage query .schema items`

---

## 常见问题诊断

### "其他"品类膨胀

```bash
sqlite3 backend/inventory.db "
SELECT t.name AS tag, COUNT(i.id) AS items
FROM items i JOIN tags t ON t.id = i.tag_id
WHERE i.category_id = 8
GROUP BY t.name ORDER BY items DESC;
"
```

### 行程物品关联完整性

```bash
sqlite3 backend/inventory.db "
SELECT
  tri.name AS trip,
  COUNT(ti.id) AS items,
  SUM(CASE WHEN ti.item_id IS NULL THEN 1 ELSE 0 END) AS orphan_slots
FROM trips tri
LEFT JOIN trip_items ti ON ti.trip_id = tri.id
GROUP BY tri.id ORDER BY tri.id;
"
```

### 属性定义分类覆盖

```bash
sqlite3 backend/inventory.db "
SELECT
  ad.key, ad.label, ad.attr_type, ad.sort_order,
  CASE WHEN ad.category_scope = '' THEN '全局' ELSE ad.category_scope END AS scope
FROM attribute_definitions ad
ORDER BY
  CASE WHEN ad.category_scope = '' THEN 1 ELSE 0 END,
  ad.category_scope, ad.sort_order;
"
```

---

## 命令 7: 物品属性智能补全 (`enrich`)

通过联网搜索产品规格 + LLM 提取属性值，批量补全物品库中缺失的属性。

### 使用方式

```
/db-manage enrich              # 全量扫描并补全
/db-manage enrich --limit 5    # 限制处理 5 个物品
/db-manage enrich --dry-run    # 仅扫描预览，不写入
```

### 完整流程

#### Phase 1: 扫描缺失属性

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS name,
       json_extract(i.attrs, '$.brand') AS brand,
       json_extract(i.attrs, '$.model') AS model,
       c.name AS cat_name, i.category_id,
       ad.key AS missing_key, ad.label AS missing_label, ad.attr_type
FROM items i
JOIN categories c ON c.id = i.category_id
JOIN attribute_definitions ad ON ad.category_scope <> '' AND ad.category_scope <> '[]'
WHERE ',' || ad.category_scope || ',' LIKE '%,' || i.category_id || ',%'
  AND (json_extract(i.attrs, '$.' || ad.key) IS NULL
       OR json_extract(i.attrs, '$.' || ad.key) = ''
       OR json_extract(i.attrs, '$.' || ad.key) = '0')
  AND ad.key NOT IN ('name','brand','model','notes','default_qty')
ORDER BY i.category_id, i.id, ad.sort_order;
"
```

输出统计摘要：各品类有多少物品、多少缺失属性。

#### Phase 2: 按物品分组 + 分批搜索

将缺失属性按物品分组。对每批物品（`--limit N` 控制批次大小，默认 10），用 WebSearch 搜索产品规格：

搜索词策略：
- 精确款：`"{brand} {model} {name} 规格参数 重量 材质"`
- 品牌近义：迪卡侬子品牌（QUECHUA/FORCLAZ/SIMOND/KIPRUN/VAN RYSEL）→ 搜索词中加"迪卡侬"
- 补搜索：首次搜索结果不足时，追加 `"{brand} {model} 评测 规格"`

搜索结果合并后进入 Phase 3。

#### Phase 3: LLM 从搜索结果提取属性值

基于搜索结果和 LLM 知识，提取每个缺失属性的值：

**属性值推断规则：**

| 属性 | 类型 | 推断规则 |
|------|------|----------|
| weight_grams | number | 从规格中找重量(g)，已知近似款可推断 |
| material | text | 从材质信息提取，如"GORE-TEX 3L"、"美利奴羊毛" |
| body_parts | text | 从产品名/用途推断部位 |
| body_parts_secondary | text | 副覆盖部位 |
| warmth_rating | bar(0-50) | 从充绒量/保温描述推断 |
| waterproof | bool(0/1) | GORE-TEX/防水膜→1，明确防水→1 |
| breathable | bool(0/1) | 速干/透气面料→1 |
| encumbrance | bar(0-10) | 拖鞋→1，重装鞋→8，T恤→2 |
| env_protection | stars(0-5) | 防风/防晒/防雨综合评分 |
| durability | stars(0-5) | 材质/用途（攀岩装备→5，速干T恤→2）|
| season | text | 从适用场景推断 |
| food_type | text | 能量胶/能量棒/巧克力/零食/饮品/调味品/主食 |
| electronics_type | text | 手机/平板/手表/耳机/充电器/数据线/电池/存储/其他 |
| storage | text | 存储容量(GB/TB) |
| ram | text | 内存(GB) |
| storage_ml | number | 容量(ml) |

**输出格式：** 每个物品+属性一个推断结果，含来源标注（"官网规格"/"评测文章"/"LLM推测"/"同类产品推断"）。

#### Phase 4: 预览

展示所有推断结果的表格：

| # | 物品 | 品类 | 缺失属性 | 建议值 | 来源 |
|---|------|------|----------|--------|------|

统计摘要：共 N 个物品，M 个属性待补充。

低置信度（LLM推测）结果标注 ⚠️。

#### Phase 5: 用户确认 + 批量写入

预览后询问用户：
- **apply** — 应用所有建议（含低置信度）
- **apply-high** — 仅应用高置信度（排除 ⚠️ LLM推测）
- **review** — 逐物品确认
- **skip** — 不写入

执行时用单条 UPDATE 写入每个物品的所有补全属性：

```sql
BEGIN TRANSACTION;
UPDATE items SET attrs = json_set(attrs, '$.weight_grams', 375, '$.waterproof', 1, '$.material', 'GORE-TEX 3L') WHERE id = 1;
-- ...
COMMIT;
```

#### Phase 6: 验证

写入后重新运行 Phase 1 扫描查询，确认缺失数量减少。

### 搜索策略优化

1. **迪卡侬子品牌映射**：QUECHUA/FORCLAZ/SIMOND/KIPRUN/VAN RYSEL/INESIS/DOMYOS/BTWIN/ROCKRIDER 都是迪卡侬旗下，搜索词中补"迪卡侬"提高召回率
2. **同品类合并**：连续同品类物品可尝试合并搜索（对比文章）
3. **缓存意识**：已在对话中搜索过的产品不重复搜索
4. **分批执行**：`--limit N` 每批处理 N 个物品，避免对话过长。默认 10

### 注意事项

- WebSearch 结果可能不精确，LLM 推断值仅作参考
- 低置信度值标注来源为"LLM推测"，用户可选择跳过
- 属性值写入前会做基本的类型校验（number/boolean 类型）
- **不删除数据库**，只做 UPDATE attrs

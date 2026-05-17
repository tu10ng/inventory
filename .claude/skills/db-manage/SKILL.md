---
name: db-manage
description: >
  Inventory 项目 SQLite 数据库管理。用于检查数据库健康状态、数据质量、
  属性 scope 覆盖、迁移状态、执行清理操作。当用户提到数据库、db、SQLite、
  数据检查、迁移、migration、scope、清理、数据质量等关键词时触发。
user-invocable: true
disable-model-invocable: false
argument-hint: "[health|scope|migrations|quality|tree|cleanup|query|enrich]"
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
SELECT 'types', COUNT(*) FROM types UNION ALL
SELECT 'items', COUNT(*) FROM items UNION ALL
SELECT 'attribute_definitions', COUNT(*) FROM attribute_definitions UNION ALL
SELECT 'status_definitions', COUNT(*) FROM status_definitions UNION ALL
SELECT 'display_rules', COUNT(*) FROM display_rules UNION ALL
SELECT 'relation_types', COUNT(*) FROM relation_types UNION ALL
SELECT 'item_relations', COUNT(*) FROM item_relations UNION ALL
SELECT 'activities', COUNT(*) FROM activities UNION ALL
SELECT 'activity_slots', COUNT(*) FROM activity_slots UNION ALL
SELECT 'activity_slot_types', COUNT(*) FROM activity_slot_types UNION ALL
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

4. 类型树统计（层级深度、父子关系）：

```bash
sqlite3 backend/inventory.db "
SELECT
  COUNT(*) AS 总类型数,
  COUNT(DISTINCT CASE WHEN parent_id IS NULL THEN id END) AS 根类型数,
  COUNT(DISTINCT CASE WHEN parent_id IS NOT NULL THEN id END) AS 子类型数,
  COUNT(DISTINCT parent_id) AS 有子节点的类型数,
  MAX(depth) AS 最大深度
FROM (
  WITH RECURSIVE depth_calc AS (
    SELECT id, parent_id, 1 AS depth FROM types WHERE parent_id IS NULL
    UNION ALL
    SELECT t.id, t.parent_id, dc.depth + 1
    FROM types t JOIN depth_calc dc ON dc.id = t.parent_id
  )
  SELECT id, parent_id, depth FROM depth_calc
);
"
```

### 输出格式

用表格展示各表行数，标注异常（如空表、行数异常多/少）。类型树统计显示层级结构和深度是否合理。

---

## 命令 2: 属性 Scope 检查 (`scope`)

检查 `attribute_definitions` 的 `category_scope` 和 `type_scope` 设置是否合理，以及物品 attrs 中是否有越界属性。

Scope 有两个维度：
- **category_scope**：限制属性适用于哪些分类（空=全局，逗号分隔的分类 ID）
- **type_scope**：限制属性适用于哪些类型（空=全局，逗号分隔的类型 ID。支持层级：scope 到父类型时子类型物品也适用）

### 执行步骤

1. 属性 category_scope 概览：

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

2. 属性 type_scope 概览：

```bash
sqlite3 backend/inventory.db "
SELECT
  CASE WHEN type_scope = '' OR type_scope IS NULL THEN '全局'
       ELSE type_scope END AS scope,
  COUNT(*) AS 数量,
  GROUP_CONCAT(key, ', ') AS 属性
FROM attribute_definitions
GROUP BY type_scope
ORDER BY scope;
"
```

3. 带 scope 属性的完整视图（同时展示 category_scope 和 type_scope）：

```bash
sqlite3 backend/inventory.db "
SELECT key, label, attr_type,
  CASE WHEN category_scope = '' THEN '全局' ELSE category_scope END AS cat_scope,
  CASE WHEN type_scope = '' THEN '全局' ELSE type_scope END AS type_scope
FROM attribute_definitions
WHERE category_scope != '' OR type_scope != ''
ORDER BY category_scope, type_scope, sort_order;
"
```

4. 检查 `category_scope = '[]'` 或 `type_scope = '[]'` 的残留（AI 创建的 bug 数据）：

```bash
sqlite3 backend/inventory.db "
SELECT key, label, category_scope, type_scope
FROM attribute_definitions
WHERE category_scope = '[]' OR type_scope = '[]';
"
```
预期：0 行。如果有结果，说明 scope 修正迁移未覆盖或 AI 后续又创建了有 bug 的属性。

5. 检查物品有无越界 category_scope（scope 非空的属性，物品 category_id 必须在 scope 内）：

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

6. 检查物品有无越界 type_scope（scope 非空的属性，物品 type_id 必须在 scope 内）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS item_name,
       i.type_id, COALESCE(t.name, '无类型') AS type_name, c.name AS cat_name,
       ad.key AS attr_key, ad.type_scope
FROM items i
JOIN categories c ON c.id = i.category_id
LEFT JOIN types t ON t.id = i.type_id
JOIN attribute_definitions ad ON ad.type_scope != '' AND ad.type_scope != '[]'
WHERE json_extract(i.attrs, '$.' || ad.key) IS NOT NULL
  AND json_extract(i.attrs, '$.' || ad.key) != ''
  AND json_extract(i.attrs, '$.' || ad.key) != '0'
  AND ',' || ad.type_scope || ',' NOT LIKE '%,' || COALESCE(i.type_id, 0) || ',%'
ORDER BY i.category_id, i.id;
"
```

### 输出

- Category scope 分布表
- Type scope 分布表
- 完整 scope 视图（同时展示两个维度）
- `'[]'` 检查结果（有/无）
- Category scope 越界物品列表（如果有）
- Type scope 越界物品列表（如果有）

---

## 命令 3: 数据质量检查 (`quality`)

全面检查数据一致性问题。

### 检查项

1. **孤儿 type**（type 的 category_id / parent_id 指向不存在的记录）：

```bash
sqlite3 backend/inventory.db "
SELECT t.id, t.name, t.category_id
FROM types t LEFT JOIN categories c ON c.id = t.category_id
WHERE c.id IS NULL;
"
```

2. **孤儿 item**（item 的 category_id/type_id 指向不存在的记录）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS name,
       i.category_id, i.type_id
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

5. **无类型物品**：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS name, c.name AS cat_name
FROM items i
JOIN categories c ON c.id = i.category_id
WHERE i.type_id IS NULL;
"
```

6. **类型分布**（每个 type 有多少物品，含子类型递归统计）：

```bash
sqlite3 backend/inventory.db "
SELECT t.name AS type, c.name AS category, COUNT(i.id) AS items
FROM types t
LEFT JOIN items i ON i.type_id = t.id
JOIN categories c ON c.id = t.category_id
GROUP BY t.id
ORDER BY items DESC;
"
```

7. **类型树完整性 — 孤儿 parent_id**（parent_id 指向不存在的类型）：

```bash
sqlite3 backend/inventory.db "
SELECT t.id, t.name, t.parent_id
FROM types t
WHERE t.parent_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM types p WHERE p.id = t.parent_id);
"
```

8. **类型树完整性 — 跨分类父子关系**（子类型的 category_id 与父类型不一致）：

```bash
sqlite3 backend/inventory.db "
SELECT c.id AS child_id, c.name AS child_name, c.category_id AS child_cat,
       p.id AS parent_id, p.name AS parent_name, p.category_id AS parent_cat
FROM types c
JOIN types p ON p.id = c.parent_id
WHERE c.category_id != p.category_id;
"
```

9. **物品挂在父类型而非子类型**（有子类型的父类型上挂了物品，应移到具体子类型）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, json_extract(i.attrs, '$.name') AS item_name,
       t.name AS type_name, c.name AS cat_name
FROM items i
JOIN types t ON t.id = i.type_id
JOIN categories c ON c.id = i.category_id
WHERE EXISTS (SELECT 1 FROM types child WHERE child.parent_id = t.id);
"
```

11. **类型递归物品统计**（含子类型后代物品，对比平面统计）：

```bash
sqlite3 backend/inventory.db "
WITH RECURSIVE type_tree AS (
  SELECT id, name, category_id, parent_id, id AS root_id, name AS root_name
  FROM types
  UNION ALL
  SELECT t.id, t.name, t.category_id, t.parent_id, tt.root_id, tt.root_name
  FROM types t
  JOIN type_tree tt ON tt.id = t.parent_id
)
SELECT tt.root_name AS type, c.name AS category,
       COUNT(DISTINCT i.id) AS total_items,  -- 含子类型后代
       (SELECT COUNT(*) FROM items WHERE type_id = tt.root_id) AS direct_items  -- 仅直接
FROM type_tree tt
JOIN categories c ON c.id = tt.category_id
LEFT JOIN items i ON i.type_id = tt.id
WHERE tt.root_id = tt.id  -- 只输出根节点
GROUP BY tt.root_id
ORDER BY total_items DESC;
"
```

### 输出

逐项展示检查结果，标注问题数量和严重程度。

---

## 命令 4: 类型树健康检查 (`tree`)

专门检查类型树形层级的完整性。

### 执行步骤

1. **类型层级概览**（每层数量）：

```bash
sqlite3 backend/inventory.db "
SELECT
  CASE WHEN parent_id IS NULL THEN '根类型'
       WHEN id IN (SELECT DISTINCT parent_id FROM types WHERE parent_id IS NOT NULL) THEN '中间类型'
       ELSE '叶子类型' END AS 层级,
  COUNT(*) AS 数量
FROM types
GROUP BY 层级
ORDER BY 层级;
"
```

2. **最大深度检查**（检测是否有过深嵌套）：

```bash
sqlite3 backend/inventory.db "
WITH RECURSIVE depth_calc AS (
  SELECT id, name, parent_id, 1 AS depth
  FROM types WHERE parent_id IS NULL
  UNION ALL
  SELECT t.id, t.name, t.parent_id, dc.depth + 1
  FROM types t JOIN depth_calc dc ON dc.id = t.parent_id
)
SELECT id, name, depth
FROM depth_calc
WHERE depth > 3
ORDER BY depth DESC, id;
"
```
预期：0 行。如果深度 > 3（如 服装 > 外套 > 冲锋衣 > 硬壳），需人工确认是否合理。

3. **循环引用检测**（parent_id 链是否形成环，需要用脚本）：

```bash
sqlite3 backend/inventory.db "
SELECT id, name, parent_id FROM types WHERE parent_id IS NOT NULL;
"
```

用脚本遍历所有 parent_id 链：从每个节点出发向上追溯，如果回到已访问节点则存在循环。

4. **各分类类型树结构一览**：

```bash
sqlite3 backend/inventory.db "
SELECT c.name AS category,
       GROUP_CONCAT(
         CASE WHEN t.parent_id IS NULL THEN t.name || '(根)'
              ELSE t.name END,
         ', '
       ) AS types
FROM types t
JOIN categories c ON c.id = t.category_id
GROUP BY t.category_id
ORDER BY c.sort_order;
"
```

---

## 命令 5: 清理越界属性 (`cleanup`)

移除物品 attrs 中不属于该分类或类型的属性值（如营养品有 `body_parts`、`warmth_rating`，或属性 type_scope 不匹配）。

### 前置条件

- migration 010 (`010_fix_category_scope.sql`) 必须已应用
- 先运行 `scope` 命令确认越界情况

### 执行步骤

1. 先预览将要清理的内容 — category_scope 越界（dry-run）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, i.category_id, c.name AS cat_name,
       json_extract(i.attrs, '$.name') AS item_name,
       ad.key AS attr_key,
       json_extract(i.attrs, '$.' || ad.key) AS attr_value,
       'category_scope 越界' AS reason
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

2. 预览 type_scope 越界（dry-run）：

```bash
sqlite3 backend/inventory.db "
SELECT i.id, i.category_id, c.name AS cat_name,
       json_extract(i.attrs, '$.name') AS item_name,
       COALESCE(t.name, '无类型') AS type_name,
       ad.key AS attr_key,
       json_extract(i.attrs, '$.' || ad.key) AS attr_value,
       'type_scope 越界' AS reason
FROM items i
JOIN categories c ON c.id = i.category_id
LEFT JOIN types t ON t.id = i.type_id
JOIN attribute_definitions ad ON ad.type_scope != '' AND ad.type_scope != '[]'
WHERE json_extract(i.attrs, '$.' || ad.key) IS NOT NULL
  AND json_extract(i.attrs, '$.' || ad.key) != ''
  AND json_extract(i.attrs, '$.' || ad.key) != '0'
  AND ',' || ad.type_scope || ',' NOT LIKE '%,' || COALESCE(i.type_id, 0) || ',%'
ORDER BY i.category_id, i.id;
"
```

3. 展示统计摘要：每个分类有多少物品、多少越界属性（category_scope + type_scope 分别统计）。
4. **必须询问用户确认**后再执行实际清理。
3. 展示统计摘要：每个分类有多少物品、多少越界属性（category_scope + type_scope 分别统计）。
4. **必须询问用户确认**后再执行实际清理。
5. 执行清理：删除 `clean_out_of_scope_attrs_done` 标记并重启后端（触发 `clean_out_of_scope_attrs()` 函数），或直接运行 SQLite JSON 操作逐物品清理。

```bash
# 方案 A: 重启触发（推荐，使用已有的 Rust 清理函数）
sqlite3 backend/inventory.db "DELETE FROM _migrations WHERE filename = '011_clean_attrs_done';"
# 然后重启后端

# 方案 B: 手动清理（逐 key 移除）
# 对于每个越界的 (item_id, key) 对执行：
# UPDATE items SET attrs = json_remove(attrs, '$.key_name') WHERE id = item_id;
```

---

## 命令 6: 迁移状态 (`migrations`)

查看迁移历史和文件对应关系。

```bash
# 已应用的迁移
sqlite3 backend/inventory.db "SELECT id, filename, applied_at FROM _migrations ORDER BY id;"

# 迁移文件列表
ls -1 backend/migrations/*.sql
```

对比迁移文件列表和 `_migrations` 表，标注未应用的文件（如果有）。

---

## 命令 7: 自定义查询 (`query`)

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
SELECT t.name AS type, COUNT(i.id) AS items
FROM items i JOIN types t ON t.id = i.type_id
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

## 命令 8: 物品属性智能补全 (`enrich`)

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

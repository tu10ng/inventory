-- Restructure categories: split overcrowded "其他" (category 8) into 家居 (6) and 服务 (7)
-- Categories 6 (洗漱) and 7 (证件) are currently empty and being repurposed

-- Phase 1: Move tags to new categories
UPDATE tags SET category_id = 6 WHERE name IN ('玩偶','购物袋','抱枕','梳子','洗衣袋','眼镜布','筷子','纸巾盒','餐具','香薰','收纳包') AND category_id = 8
;
UPDATE tags SET category_id = 7 WHERE name IN ('服务','课程') AND category_id = 8
;
UPDATE tags SET category_id = 2 WHERE name = '驱虫剂' AND category_id = 8
;

-- Phase 2: Update items' category_id based on their tag
UPDATE items SET category_id = 6 WHERE category_id = 8 AND tag_id IN (SELECT id FROM tags WHERE name IN ('玩偶','购物袋','抱枕','梳子','洗衣袋','眼镜布','筷子','纸巾盒','餐具','香薰','收纳包'))
;
UPDATE items SET category_id = 7 WHERE category_id = 8 AND tag_id IN (SELECT id FROM tags WHERE name IN ('服务','课程'))
;
UPDATE items SET category_id = 2 WHERE category_id = 8 AND tag_id IN (SELECT id FROM tags WHERE name = '驱虫剂')
;

-- Phase 3: Update activity slot references
UPDATE activity_slots SET category_id = 6 WHERE slot_name = '颈枕' AND category_id = 8
;
-- 名片保留在 8 (其他), no items matched

-- Phase 4: Rename categories
UPDATE categories SET name = '家居', icon = '🏠' WHERE id = 6
;
UPDATE categories SET name = '服务', icon = '💳' WHERE id = 7

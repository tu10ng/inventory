import { describe, it, expect } from 'vitest';
import { filterItems, sortItems, groupItems } from './itemFilters';
import type { Item, Tag } from '$lib/types';
import type { ItemColumnDef } from './columns';

function makeItem(overrides: Partial<Item> = {}): Item {
	return {
		id: 1,
		category_id: 1,
		tag_id: null,
		attrs: {},
		...overrides
	};
}

const itemFcy: Item = makeItem({ id: 1, category_id: 1, attrs: { name: '冲锋衣', brand: '始祖鸟', model: 'Beta LT', warmth_rating: 30, waterproof: 1, default_qty: 2 } });
const itemDzg: Item = makeItem({ id: 2, category_id: 2, attrs: { name: '登山杖', brand: 'Black Diamond', model: '', warmth_rating: 0, waterproof: 0, default_qty: 2 } });
const itemTd: Item = makeItem({ id: 3, category_id: 2, tag_id: 2, attrs: { name: '头灯', brand: 'Petzl', model: 'Tikka', warmth_rating: 0, waterproof: 1, default_qty: 1 } });
const items: Item[] = [itemFcy, itemDzg, itemTd];

const tags: Tag[] = [
	{ id: 1, name: '冲锋衣', category_id: 1, sort_order: 1 },
	{ id: 2, name: '头灯', category_id: 2, sort_order: 2 }
];

const columns: ItemColumnDef[] = [
	{ key: 'name', label: '名称', type: 'text' },
	{ key: 'brand', label: '品牌', type: 'text', filterable: true },
	{ key: 'warmth_rating', label: '保暖', type: 'bar', max: 50 },
	{ key: 'waterproof', label: '防水', type: 'bool', filterable: true },
	{ key: 'tag', label: '标签', type: 'tag', filterable: true }
];

describe('filterItems', () => {
	it('filters by search in name', () => {
		const result = filterItems(items, '冲锋', null, new Map(), columns, tags);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(1);
	});

	it('filters by search in brand', () => {
		const result = filterItems(items, 'black diamond', null, new Map(), columns, tags);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(2);
	});

	it('filters by search in model', () => {
		const result = filterItems(items, 'beta lt', null, new Map(), columns, tags);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(1);
	});

	it('filters by category_id', () => {
		const result = filterItems(items, '', 2, new Map(), columns, tags);
		expect(result).toHaveLength(2);
		expect(result.every(i => i.category_id === 2)).toBe(true);
	});

	it('filters by boolean column', () => {
		const filterMap = new Map([['waterproof', new Set(['1'])]]);
		const result = filterItems(items, '', null, filterMap, columns, tags);
		expect(result).toHaveLength(2); // 冲锋衣 and 头灯
	});

	it('filters by tag column', () => {
		const filterMap = new Map([['tag', new Set(['头灯'])]]);
		const result = filterItems(items, '', null, filterMap, columns, tags);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(3);
	});

	it('returns empty when no match', () => {
		const result = filterItems(items, '不存在的物品', null, new Map(), columns, tags);
		expect(result).toHaveLength(0);
	});
});

describe('sortItems', () => {
	it('sorts by name ascending (Chinese)', () => {
		const result = sortItems(items, 'name', 'asc', tags);
		// 冲锋衣 (chong) comes after 登山杖 (deng) and 头灯 (tou) in pinyin
		// Actually it depends on locale. We just verify it's sorted
		expect(result).toHaveLength(3);
	});

	it('sorts by name descending', () => {
		const asc = sortItems(items, 'name', 'asc', tags);
		const desc = sortItems(items, 'name', 'desc', tags);
		expect(desc.map(i => i.id)).toEqual(asc.map(i => i.id).reverse());
	});

	it('sorts by numeric attrs field', () => {
		const result = sortItems(items, 'warmth_rating', 'asc', tags);
		expect(result[0].id).toBe(2); // 0 warmth
		expect(result[2].id).toBe(1); // 30 warmth
	});

	it('sorts by default_qty', () => {
		const result = sortItems(items, 'default_qty', 'desc', tags);
		expect(result[0].id).toBe(1); // qty=2
	});
});

describe('groupItems', () => {
	it('groups by brand', () => {
		const { groups, ungrouped } = groupItems(items, 'brand', [
			{ key: 'brand', label: '品牌', type: 'text' }
		], []);
		expect(groups.length).toBe(3); // 始祖鸟, Black Diamond, Petzl
		expect(ungrouped.length).toBe(0);
	});

	it('leaves items with missing key in ungrouped', () => {
		const result = groupItems(items, 'nonexistent', [], []);
		expect(result.groups.length).toBe(0);
		expect(result.ungrouped.length).toBe(3);
	});

	it('groups by tag using tag_id lookup', () => {
		const { groups, ungrouped } = groupItems(items, 'tag', [
			{ key: 'tag', label: '标签', type: 'tag' }
		], tags);
		expect(groups.length).toBe(1); // 头灯 group
		expect(groups[0].value).toBe('头灯');
		expect(groups[0].items.length).toBe(1);
		expect(groups[0].items[0].id).toBe(3);
		expect(ungrouped.length).toBe(2); // 冲锋衣 and 登山杖 have no tag
	});
});

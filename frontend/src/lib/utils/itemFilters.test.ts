import { describe, it, expect } from 'vitest';
import { filterItems, sortItems, groupItems } from './itemFilters';
import type { Item, Type } from '$lib/types';
import type { ItemColumnDef } from './columns';

function makeItem(overrides: Partial<Item> = {}): Item {
	return {
		id: 1,
		type_id: null,
		attrs: {},
		...overrides
	};
}

const itemFcy: Item = makeItem({ id: 1, attrs: { name: '冲锋衣', brand: '始祖鸟', model: 'Beta LT', warmth_rating: 30, waterproof: 1, default_qty: 2 } });
const itemDzg: Item = makeItem({ id: 2, type_id: 2, attrs: { name: '登山杖', brand: 'Black Diamond', model: '', warmth_rating: 0, waterproof: 0, default_qty: 2 } });
const itemTd: Item = makeItem({ id: 3, type_id: 3, attrs: { name: '头灯', brand: 'Petzl', model: 'Tikka', warmth_rating: 0, waterproof: 1, default_qty: 1 } });
const items: Item[] = [itemFcy, itemDzg, itemTd];

const types: Type[] = [
	{ id: 1, name: '服装', sort_order: 1, parent_id: null },
	{ id: 2, name: '登山杖', sort_order: 1, parent_id: 1 },
	{ id: 3, name: '头灯', sort_order: 2, parent_id: 1 }
];

const columns: ItemColumnDef[] = [
	{ key: 'name', label: '名称', type: 'text' },
	{ key: 'brand', label: '品牌', type: 'text', filterable: true },
	{ key: 'warmth_rating', label: '保暖', type: 'bar', max: 50 },
	{ key: 'waterproof', label: '防水', type: 'bool', filterable: true },
	{ key: 'type', label: '类型', type: 'type', filterable: true }
];

describe('filterItems', () => {
	it('filters by search in name', () => {
		const result = filterItems(items, '冲锋', null, new Map(), columns, types);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(1);
	});

	it('filters by search in brand', () => {
		const result = filterItems(items, 'black diamond', null, new Map(), columns, types);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(2);
	});

	it('filters by search in model', () => {
		const result = filterItems(items, 'beta lt', null, new Map(), columns, types);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(1);
	});

	it('filters by root type', () => {
		// Both itemDzg (type 2, root 1) and itemTd (type 3, root 1) have root type 1
		const result = filterItems(items, '', 1, new Map(), columns, types);
		expect(result).toHaveLength(2);
		expect(result.map(i => i.id).sort()).toEqual([2, 3]);
	});

	it('filters by boolean column', () => {
		const filterMap = new Map([['waterproof', new Set(['1'])]]);
		const result = filterItems(items, '', null, filterMap, columns, types);
		expect(result).toHaveLength(2); // 冲锋衣 and 头灯
	});

	it('filters by type column', () => {
		const filterMap = new Map([['type', new Set(['3'])]]);
		const result = filterItems(items, '', null, filterMap, columns, types);
		expect(result).toHaveLength(1);
		expect(result[0].id).toBe(3);
	});

	it('returns empty when no match', () => {
		const result = filterItems(items, '不存在的物品', null, new Map(), columns, types);
		expect(result).toHaveLength(0);
	});
});

describe('sortItems', () => {
	it('sorts by name ascending (Chinese)', () => {
		const result = sortItems(items, 'name', 'asc', types);
		expect(result).toHaveLength(3);
	});

	it('sorts by name descending', () => {
		const asc = sortItems(items, 'name', 'asc', types);
		const desc = sortItems(items, 'name', 'desc', types);
		expect(desc.map(i => i.id)).toEqual(asc.map(i => i.id).reverse());
	});

	it('sorts by numeric attrs field', () => {
		const result = sortItems(items, 'warmth_rating', 'asc', types);
		expect(result[0].id).toBe(2); // 0 warmth
		expect(result[2].id).toBe(1); // 30 warmth
	});

	it('sorts by default_qty', () => {
		const result = sortItems(items, 'default_qty', 'desc', types);
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

	it('groups by type using type_id lookup', () => {
		const { groups, ungrouped } = groupItems(items, 'type', [
			{ key: 'type', label: '类型', type: 'type' }
		], types);
		expect(groups.length).toBe(2); // 登山杖 and 头灯 groups
		const values = groups.map(g => g.value);
		expect(values).toContain('登山杖');
		expect(values).toContain('头灯');
		expect(ungrouped.length).toBe(1); // 冲锋衣 has no type_id
	});
});

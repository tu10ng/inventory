import { describe, it, expect } from 'vitest';
import { getCellValue } from './cellValue';
import type { Item } from '$lib/types';
import type { ItemColumnDef } from './columns';

describe('getCellValue', () => {
	const item: Item = {
		id: 1,
		category_id: 1,
		type_id: null,
		attrs: { name: '冲锋衣', brand: '始祖鸟', warmth_rating: 30, waterproof: 1 }
	};

	const nameCol: ItemColumnDef = { key: 'name', label: '名称', type: 'text' };
	const brandCol: ItemColumnDef = { key: 'brand', label: '品牌', type: 'text' };
	const warmthCol: ItemColumnDef = { key: 'warmth_rating', label: '保暖', type: 'bar' };
	const missingCol: ItemColumnDef = { key: 'nonexistent', label: '不存在', type: 'text' };

	it('returns attrs.name value', () => {
		expect(getCellValue(item, nameCol)).toBe('冲锋衣');
	});

	it('returns attrs.brand value', () => {
		expect(getCellValue(item, brandCol)).toBe('始祖鸟');
	});

	it('returns numeric attrs value', () => {
		expect(getCellValue(item, warmthCol)).toBe(30);
	});

	it('returns null for missing key', () => {
		expect(getCellValue(item, missingCol)).toBeNull();
	});

	it('returns null when attrs is empty', () => {
		const empty: Item = { id: 2, category_id: 1, type_id: null, attrs: {} };
		expect(getCellValue(empty, nameCol)).toBeNull();
	});
});

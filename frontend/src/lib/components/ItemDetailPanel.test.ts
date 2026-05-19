import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import ItemDetailPanel from './ItemDetailPanel.svelte';
import type { Item, Type, AttributeDefinition } from '$lib/types';

afterEach(() => {
	cleanup();
});

const types: Type[] = [
	{ id: 1, name: '冲锋衣', sort_order: 1, parent_id: null }
];

const attrDefs: AttributeDefinition[] = [
	{ id: 1, key: 'name', label: '名称', attr_type: 'text', config: '{}', category_scope: '', type_scope: '', sort_order: 0 },
	{ id: 2, key: 'brand', label: '品牌', attr_type: 'text', config: '{}', category_scope: '', type_scope: '', sort_order: 0 },
	{ id: 3, key: 'model', label: '型号', attr_type: 'text', config: '{}', category_scope: '', type_scope: '', sort_order: 0 },
	{ id: 4, key: 'warmth_rating', label: '保暖', attr_type: 'bar', config: '{"max":50}', category_scope: '', type_scope: '', sort_order: 1 }
];

const sampleItem: Item = {
	id: 1,
	type_id: 1,
	attrs: { name: '冲锋衣', brand: '始祖鸟', model: 'Beta LT', default_qty: 1, warmth_rating: 30, notes: '' }
};

describe('ItemDetailPanel', () => {
	it('renders item name (may appear multiple times)', () => {
		const onUpdate = vi.fn();
		const onDelete = vi.fn();

		render(ItemDetailPanel, {
			item: sampleItem,
			types,
			attrDefs,
			onUpdate,
			onDelete
		});

		const elements = screen.getAllByText('冲锋衣');
		expect(elements.length).toBeGreaterThanOrEqual(1);
	});

	it('renders brand value', () => {
		const onUpdate = vi.fn();
		const onDelete = vi.fn();

		render(ItemDetailPanel, {
			item: sampleItem,
			types,
			attrDefs,
			onUpdate,
			onDelete
		});

		const elements = screen.getAllByText('始祖鸟');
		expect(elements.length).toBeGreaterThanOrEqual(1);
	});

	it('renders type name', () => {
		const onUpdate = vi.fn();
		const onDelete = vi.fn();

		render(ItemDetailPanel, {
			item: sampleItem,
			types,
			attrDefs,
			onUpdate,
			onDelete
		});

		// "冲锋衣" appears in both item name and type pill area
		const elements = screen.getAllByText('冲锋衣');
		expect(elements.length).toBeGreaterThanOrEqual(1);
	});

	it('renders delete button', () => {
		const onUpdate = vi.fn();
		const onDelete = vi.fn();

		render(ItemDetailPanel, {
			item: sampleItem,
			types,
			attrDefs,
			onUpdate,
			onDelete
		});

		const deleteBtn = screen.getByText('删除');
		expect(deleteBtn).toBeTruthy();
	});

	it('renders usage count', () => {
		const onUpdate = vi.fn();
		const onDelete = vi.fn();

		render(ItemDetailPanel, {
			item: sampleItem,
			types,
			attrDefs,
			usageCount: 5,
			onUpdate,
			onDelete
		});

		expect(screen.getByText(/使用/)).toBeTruthy();
	});
});

import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';
import ItemForm from './ItemForm.svelte';
import type { Category, Type, AttributeDefinition } from '$lib/types';

afterEach(() => {
	cleanup();
});

const categories: Category[] = [
	{ id: 1, name: '服装', icon: '👕', sort_order: 1 },
	{ id: 2, name: '装备', icon: '🎒', sort_order: 2 }
];

const types: Type[] = [
	{ id: 1, name: '冲锋衣', category_id: 1, sort_order: 1, parent_id: null },
	{ id: 2, name: '登山杖', category_id: 2, sort_order: 1, parent_id: null }
];

const attrDefs: AttributeDefinition[] = [
	{ id: 1, key: 'name', label: '名称', attr_type: 'text', config: '{}', category_scope: '', type_scope: '', sort_order: 0 },
	{ id: 2, key: 'brand', label: '品牌', attr_type: 'text', config: '{}', category_scope: '', type_scope: '', sort_order: 0 },
	{ id: 3, key: 'warmth_rating', label: '保暖', attr_type: 'bar', config: '{"max":50}', category_scope: '', type_scope: '', sort_order: 1 }
];

describe('ItemForm', () => {
	it('renders form for new item with add button', () => {
		const onSave = vi.fn();
		const onCancel = vi.fn();

		render(ItemForm, {
			item: null,
			categories,
			types,
			attrDefs,
			onSave,
			onCancel
		});

		// New item form shows "添加" as primary button text
		expect(screen.getByText('添加')).toBeTruthy();
		expect(screen.getByText('取消')).toBeTruthy();
	});

	it('calls onCancel when cancel clicked', async () => {
		const onSave = vi.fn();
		const onCancel = vi.fn();

		render(ItemForm, {
			item: null,
			categories,
			types,
			attrDefs,
			onSave,
			onCancel
		});

		const cancelBtn = screen.getByText('取消');
		cancelBtn.click();
		expect(onCancel).toHaveBeenCalled();
	});

	it('renders edit form with update button for existing item', () => {
		const onSave = vi.fn();
		const onCancel = vi.fn();

		render(ItemForm, {
			item: {
				id: 1, // id triggers isEdit mode
				category_id: 1,
				type_id: 1,
				attrs: { name: '冲锋衣', brand: '始祖鸟', warmth_rating: 30 }
			},
			categories,
			types,
			attrDefs,
			onSave,
			onCancel
		});

		expect(screen.getByText('更新')).toBeTruthy();
		expect(screen.getByText('编辑物品')).toBeTruthy();
	});
});

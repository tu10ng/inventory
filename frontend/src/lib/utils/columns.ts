import type { ItemColumnDef } from '$lib/types';

const STORAGE_KEY = 'inventory-visible-columns';

export const ALL_COLUMNS: ItemColumnDef[] = [
	{
		key: 'name',
		label: '名称',
		width: '1fr',
		render: 'text',
		getValue: (item) => item.name,
		sortable: true,
	},
	{
		key: 'tag',
		label: '标签',
		width: '80px',
		render: 'tag',
		getValue: (item, ctx) => {
			if (!item.tag_id || !ctx?.tags) return '';
			return ctx.tags.find((t) => t.id === item.tag_id)?.name ?? '';
		},
	},
	{
		key: 'brand',
		label: '品牌',
		width: '90px',
		render: 'text',
		getValue: (item) => item.brand,
	},
	{
		key: 'model',
		label: '型号',
		width: '90px',
		render: 'text',
		getValue: (item) => item.model,
	},
	{
		key: 'weight',
		label: '重量',
		width: '70px',
		render: 'weight',
		getValue: (item) => item.weight_grams,
		sortable: true,
	},
	{
		key: 'warmth',
		label: '保暖',
		width: '100px',
		render: 'bar',
		getValue: (item) => item.warmth_rating,
		sortable: true,
	},
	{
		key: 'encumbrance',
		label: '累赘',
		width: '70px',
		render: 'number',
		getValue: (item) => item.encumbrance,
		sortable: true,
	},
	{
		key: 'waterproof',
		label: '防水',
		width: '50px',
		render: 'bool',
		getValue: (item) => item.waterproof > 0,
	},
	{
		key: 'breathable',
		label: '透气',
		width: '50px',
		render: 'bool',
		getValue: (item) => item.breathable > 0,
	},
	{
		key: 'env_protection',
		label: '环境防护',
		width: '90px',
		render: 'stars',
		getValue: (item) => item.env_protection,
		sortable: true,
	},
	{
		key: 'durability',
		label: '耐久',
		width: '90px',
		render: 'stars',
		getValue: (item) => item.durability,
		sortable: true,
	},
	{
		key: 'usage',
		label: '使用',
		width: '60px',
		render: 'number',
		getValue: (item, ctx) => ctx?.usageStats?.get(item.id) ?? 0,
		sortable: true,
	},
];

const DEFAULT_VISIBLE = ['name', 'tag', 'brand', 'model', 'weight', 'warmth', 'waterproof'];

export function loadVisibleColumns(): string[] {
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const keys = JSON.parse(stored) as string[];
			if (Array.isArray(keys) && keys.length > 0) {
				// Always include 'name'
				if (!keys.includes('name')) keys.unshift('name');
				return keys;
			}
		}
	} catch {
		// ignore
	}
	return DEFAULT_VISIBLE;
}

export function saveVisibleColumns(keys: string[]) {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(keys));
}

import type { ItemStatus } from '$lib/types';

export const STATUS_LABELS: Record<ItemStatus, string> = {
	'': '',
	need_buy: '需购买',
	need_find: '需寻找',
	need_charge: '需充电',
	need_fetch: '需取回',
	need_give: '需带给'
};

export const STATUS_OPTIONS: { value: ItemStatus; label: string }[] = [
	{ value: '', label: '无' },
	{ value: 'need_buy', label: '需购买' },
	{ value: 'need_find', label: '需寻找' },
	{ value: 'need_charge', label: '需充电' },
	{ value: 'need_fetch', label: '需取回' },
	{ value: 'need_give', label: '需带给' }
];

export const TRIP_STATUS_LABELS: Record<string, string> = {
	planning: '计划中',
	packing: '打包中',
	done: '已完成'
};

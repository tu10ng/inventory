export interface ItemColumnDef {
	key: string;
	label: string;
	type: 'text' | 'number' | 'bool' | 'bar' | 'stars' | 'tag' | 'weight';
	suffix?: string;
	max?: number;
	sortable?: boolean;
	filterable?: boolean;
}

export const ALL_COLUMNS: ItemColumnDef[] = [
	{ key: 'tag', label: '标签', type: 'tag', filterable: true },
	{ key: 'brand', label: '品牌', type: 'text', filterable: true },
	{ key: 'weight_grams', label: '重量', type: 'weight', suffix: 'g', filterable: false },
	{ key: 'warmth_rating', label: '保暖', type: 'bar', max: 50, filterable: false },
	{ key: 'encumbrance', label: '累赘', type: 'bar', max: 10, filterable: false },
	{ key: 'env_protection', label: '环境防护', type: 'stars', filterable: false },
	{ key: 'durability', label: '耐久', type: 'stars', filterable: false },
	{ key: 'waterproof', label: '防水', type: 'bool', filterable: true },
	{ key: 'breathable', label: '透气', type: 'bool', filterable: true },
	{ key: 'default_qty', label: '数量', type: 'number', filterable: false },
	{ key: 'storage_ml', label: '容量', type: 'number', suffix: 'ml', filterable: false },
];

const STORAGE_KEY = 'inventory-visible-columns';
const DEFAULT_KEYS = ['tag', 'brand'];

export function loadVisibleColumns(): string[] {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) {
			const parsed = JSON.parse(raw);
			if (Array.isArray(parsed) && parsed.length > 0) return parsed;
		}
	} catch {
		// ignore
	}
	return [...DEFAULT_KEYS];
}

export function saveVisibleColumns(keys: string[]): void {
	localStorage.setItem(STORAGE_KEY, JSON.stringify(keys));
}

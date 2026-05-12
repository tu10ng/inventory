import { api } from '$lib/api/client';
import type { AttributeDefinition } from '$lib/types';

export interface ItemColumnDef {
	key: string;
	label: string;
	type: 'text' | 'number' | 'bool' | 'bar' | 'stars' | 'tag' | 'weight';
	suffix?: string;
	max?: number;
	sortable?: boolean;
	filterable?: boolean;
}

let cachedColumns: ItemColumnDef[] | null = null;

function attrDefToColumn(ad: AttributeDefinition): ItemColumnDef {
	const config = ad.config ? JSON.parse(ad.config) : {};
	const col: ItemColumnDef = {
		key: ad.key,
		label: ad.label,
		type: ad.attr_type as ItemColumnDef['type'],
		filterable: ad.attr_type === 'text' || ad.attr_type === 'bool' || ad.attr_type === 'tag',
	};
	if (config.max) col.max = config.max;
	if (config.suffix) col.suffix = config.suffix;
	return col;
}

/** Check if an attribute definition matches the given category/tag scope. */
export function attrMatchesScope(
	ad: AttributeDefinition,
	categoryId: number | null,
	tagId: number | null
): boolean {
	const catScope = ad.category_scope
		? ad.category_scope.split(',').filter(Boolean).map(Number)
		: [];
	const tagScope = ad.tag_scope
		? ad.tag_scope.split(',').filter(Boolean).map(Number)
		: [];

	// Global: both scopes empty
	if (catScope.length === 0 && tagScope.length === 0) return true;

	let ok = false;
	if (catScope.length > 0 && categoryId != null) {
		ok = ok || catScope.includes(categoryId);
	}
	if (tagScope.length > 0 && tagId != null) {
		ok = ok || tagScope.includes(tagId);
	}
	return ok;
}

export async function loadAllColumns(): Promise<ItemColumnDef[]> {
	if (cachedColumns) return cachedColumns;
	const defs = await api.get<AttributeDefinition[]>('/attribute-definitions');
	// Prepend the tag column (special, not an attribute)
	// All others come from attribute_definitions, including name, brand, model
	cachedColumns = [
		{ key: 'tag', label: '标签', type: 'tag', filterable: true },
		...defs.map(attrDefToColumn),
	];
	return cachedColumns;
}

export function invalidateColumnsCache(): void {
	cachedColumns = null;
}

// Synchronous access after initial load
export function getAllColumns(): ItemColumnDef[] {
	return cachedColumns ?? [];
}

const STORAGE_KEY = 'inventory-visible-columns';
const DEFAULT_KEYS = ['tag', 'name'];

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

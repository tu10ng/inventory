import { api } from '$lib/api/client';
import type { AttributeDefinition, Type } from '$lib/types';

export interface ItemColumnDef {
	key: string;
	label: string;
	type: 'text' | 'number' | 'bool' | 'bar' | 'stars' | 'type' | 'weight';
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
		filterable: ad.attr_type === 'text' || ad.attr_type === 'bool' || ad.attr_type === 'type',
	};
	if (config.max) col.max = config.max;
	if (config.suffix) col.suffix = config.suffix;
	return col;
}

/** Check if an attribute definition matches the given category/type scope. */
export function attrMatchesScope(
	ad: AttributeDefinition,
	categoryId: number | null,
	typeId: number | null
): boolean {
	const catScope = ad.category_scope
		? ad.category_scope.split(',').filter(Boolean).map(Number)
		: [];
	const typeScope = ad.type_scope
		? ad.type_scope.split(',').filter(Boolean).map(Number)
		: [];

	// Global: both scopes empty
	if (catScope.length === 0 && typeScope.length === 0) return true;

	let ok = false;
	if (catScope.length > 0 && categoryId != null) {
		ok = ok || catScope.includes(categoryId);
	}
	if (typeScope.length > 0 && typeId != null) {
		ok = ok || typeScope.includes(typeId);
	}
	return ok;
}

export async function loadAllColumns(): Promise<ItemColumnDef[]> {
	if (cachedColumns) return cachedColumns;
	const defs = await api.get<AttributeDefinition[]>('/attribute-definitions');
	// Prepend the type column (special, not an attribute)
	// All others come from attribute_definitions, including name, brand, model
	cachedColumns = [
		{ key: 'type', label: '类型', type: 'type', filterable: true },
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
const DEFAULT_KEYS = ['type', 'name'];

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

/** Build full path string from type_id to root, e.g. "服装 > 外套 > 冲锋衣" */
export function buildTypePath(typeId: number | null | undefined, types: Type[]): string {
	if (typeId == null) return '无类型';
	const path: string[] = [];
	let current: Type | undefined = types.find(t => t.id === typeId);
	while (current) {
		path.unshift(current.name);
		current = current.parent_id ? types.find(t => t.id === current!.parent_id) : undefined;
	}
	return path.join(' > ');
}

/** Get all descendant type IDs (including the parent itself) for recursive filtering */
export function getDescendantTypeIds(parentId: number, types: Type[]): Set<number> {
	const descendants = new Set<number>();
	descendants.add(parentId);
	const children = types.filter(t => t.parent_id === parentId);
	for (const child of children) {
		for (const id of getDescendantTypeIds(child.id, types)) {
			descendants.add(id);
		}
	}
	return descendants;
}

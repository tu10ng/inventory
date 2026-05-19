import type { Item, Type } from '$lib/types';
import type { ItemColumnDef } from '$lib/utils/columns';
import { getDescendantTypeIds, getRootTypeId } from '$lib/utils/columns';

export interface ItemGroup {
	label: string;
	value: string;
	items: Item[];
}

export interface TypeTreeGroup {
	type: Type;
	items: Item[];
	children: TypeTreeGroup[];
}

export function filterItems(
	items: Item[],
	search: string,
	filterRootTypeId: number | null,
	columnFilters: Map<string, Set<string>>,
	allColumns: ItemColumnDef[],
	types: Type[]
): Item[] {
	let list = items;
	if (search) {
		const q = search.toLowerCase();
		list = list.filter((i) => {
			// Search in name, brand, model (identity fields via attrs)
			const name = String(i.attrs?.name ?? '').toLowerCase();
			const brand = String(i.attrs?.brand ?? '').toLowerCase();
			const model = String(i.attrs?.model ?? '').toLowerCase();
			return name.includes(q) || brand.includes(q) || model.includes(q);
		});
	}
	if (filterRootTypeId !== null) {
		list = list.filter((i) => getRootTypeId(i.type_id, types) === filterRootTypeId);
	}
	// Apply column filters
	for (const [key, vals] of columnFilters) {
		if (vals.size === 0) continue;
		const col = allColumns.find(c => c.key === key);
		if (!col) continue;
		list = list.filter((item) => {
			if (col.type === 'type') {
				// For type column, use recursive matching (parent matches children)
				const descendantIds = new Set<number>();
				for (const val of vals) {
					const id = Number(val);
					if (!isNaN(id)) {
						for (const did of getDescendantTypeIds(id, types)) {
							descendantIds.add(did);
						}
					}
				}
				if (!descendantIds.has(item.type_id ?? 0)) {
					return false;
				}
				return true;
			} else if (col.type === 'bool') {
				const v = item.attrs?.[key] as number;
				return vals.has(v > 0 ? '1' : '0');
			} else {
				const v = item.attrs?.[key];
				return vals.has(v ? String(v) : '-');
			}
		});
	}
	return list;
}

export function sortItems(
	items: Item[],
	sortKey: string | null,
	sortDir: 'asc' | 'desc',
	types: Type[]
): Item[] {
	if (!sortKey) return items;
	const key = sortKey;
	const dir = sortDir;
	return [...items].sort((a, b) => {
		let va: unknown, vb: unknown;
		if (key === 'type') {
			const ta = a.type_id ? types.find(t => t.id === a.type_id) : null;
			const tb = b.type_id ? types.find(t => t.id === b.type_id) : null;
			va = ta?.name ?? '';
			vb = tb?.name ?? '';
		} else {
			va = a.attrs?.[key];
			vb = b.attrs?.[key];
		}
		// Nullish values go last
		if (va == null && vb == null) return 0;
		if (va == null) return 1;
		if (vb == null) return -1;
		let cmp: number;
		if (typeof va === 'string' && typeof vb === 'string') {
			cmp = va.localeCompare(vb, 'zh');
		} else {
			cmp = Number(va) - Number(vb);
		}
		return dir === 'asc' ? cmp : -cmp;
	});
}

export function groupItems(
	items: Item[],
	groupByKey: string,
	columns: ItemColumnDef[],
	types: Type[]
): { groups: ItemGroup[]; ungrouped: Item[] } {
	const col = columns.find(c => c.key === groupByKey);
	const label = col?.label ?? groupByKey;

	const groupsMap = new Map<string, Item[]>();
	const ungrouped: Item[] = [];

	for (const item of items) {
		let value: unknown;
		if (groupByKey === 'type') {
			const t = item.type_id ? types.find(tg => tg.id === item.type_id) : null;
			value = t?.name ?? null;
		} else {
			value = item.attrs?.[groupByKey];
		}
		if (value != null && String(value).trim() !== '') {
			const key = String(value).trim();
			if (!groupsMap.has(key)) {
				groupsMap.set(key, []);
			}
			groupsMap.get(key)!.push(item);
		} else {
			ungrouped.push(item);
		}
	}

	const groups: ItemGroup[] = [];
	for (const [value, groupItems] of groupsMap) {
		groups.push({
			label: `${label}: ${value}`,
			value,
			items: groupItems
		});
	}

	// Sort groups by label
	groups.sort((a, b) => a.label.localeCompare(b.label, 'zh'));

	return { groups, ungrouped };
}

/** Build a tree of TypeTreeGroup from items, grouped by root type (parent_id == null).
 *  Items with no type_id go to ungrouped. */
export function groupItemsByTypeTree(
	items: Item[],
	types: Type[]
): Map<number, { tree: TypeTreeGroup[]; ungrouped: Item[] }> {
	const typeMap = new Map(types.map(t => [t.id, t]));

	// Map type_id → items
	const typeItems = new Map<number, Item[]>();
	const ungrouped: Item[] = [];

	for (const item of items) {
		if (item.type_id != null && typeMap.has(item.type_id)) {
			if (!typeItems.has(item.type_id)) {
				typeItems.set(item.type_id, []);
			}
			typeItems.get(item.type_id)!.push(item);
		} else {
			ungrouped.push(item);
		}
	}

	// Group items by root type
	const rootMap = new Map<number, { items: Item[]; typeIds: Set<number> }>();

	for (const item of items) {
		if (item.type_id == null) continue;
		const rootId = getRootTypeId(item.type_id, types);
		if (rootId == null) continue;

		if (!rootMap.has(rootId)) {
			rootMap.set(rootId, { items: [], typeIds: new Set() });
		}
		rootMap.get(rootId)!.items.push(item);
		const t = typeMap.get(item.type_id);
		if (t) {
			// Also collect all ancestor type IDs along the chain
			let current: Type | undefined = t;
			while (current) {
				rootMap.get(rootId)!.typeIds.add(current.id);
				current = current.parent_id ? typeMap.get(current.parent_id) : undefined;
			}
		}
	}

	const result = new Map<number, { tree: TypeTreeGroup[]; ungrouped: Item[] }>();

	for (const [rootId, { items: rootItems, typeIds }] of rootMap) {
		// Track which types are already included in the tree to catch orphans later
		const inTree = new Set<number>();

		function buildNode(type: Type): TypeTreeGroup {
			inTree.add(type.id);
			const node: TypeTreeGroup = {
				type,
				items: typeItems.get(type.id) ?? [],
				children: [],
			};
			const childTypes = types.filter(t => t.parent_id === type.id && typeIds.has(t.id));
			for (const child of childTypes) {
				const childNode = buildNode(child);
				// Only include child if it has items (direct or via descendants)
				if (childNode.items.length > 0 || childNode.children.length > 0) {
					node.children.push(childNode);
				}
			}
			return node;
		}

		// Root type
		const rootType = typeMap.get(rootId)!;
		const tree = [buildNode(rootType)].filter(n => n.items.length > 0 || n.children.length > 0);

		// Handle orphaned types with items that weren't reachable from root
		for (const [typeId, its] of typeItems) {
			if (!inTree.has(typeId) && its.length > 0 && getRootTypeId(typeId, types) === rootId) {
				const type = typeMap.get(typeId)!;
				tree.push({ type, items: its, children: [] });
			}
		}

		// Collect unassigned items for this root
		const thisUngrouped = rootItems.filter(i => i.type_id == null || !typeMap.has(i.type_id));

		result.set(rootId, { tree, ungrouped: thisUngrouped });
	}

	// Also handle global ungrouped items (those not in any root group)
	const itemsWithoutRoot = items.filter(i => {
		if (i.type_id == null) return true;
		return getRootTypeId(i.type_id, types) == null;
	});
	if (itemsWithoutRoot.length > 0 && !result.has(-1)) {
		result.set(-1, { tree: [], ungrouped: itemsWithoutRoot });
	}

	return result;
}

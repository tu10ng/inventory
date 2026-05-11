import type { Item, Tag } from '$lib/types';
import type { ItemColumnDef } from '$lib/utils/columns';

export function filterItems(
	items: Item[],
	search: string,
	filterCategoryId: number | null,
	columnFilters: Map<string, Set<string>>,
	allColumns: ItemColumnDef[],
	tags: Tag[]
): Item[] {
	let list = items;
	if (search) {
		const q = search.toLowerCase();
		list = list.filter(
			(i) =>
				i.name.toLowerCase().includes(q) ||
				i.brand.toLowerCase().includes(q) ||
				i.model.toLowerCase().includes(q)
		);
	}
	if (filterCategoryId !== null) {
		list = list.filter((i) => i.category_id === filterCategoryId);
	}
	// Apply column filters
	for (const [key, vals] of columnFilters) {
		if (vals.size === 0) continue;
		const col = allColumns.find(c => c.key === key);
		if (!col) continue;
		list = list.filter((item) => {
			if (col.type === 'tag') {
				const t = item.tag_id ? tags.find(tg => tg.id === item.tag_id) : null;
				const display = t ? t.name : '-';
				return vals.has(display);
			} else if (col.type === 'bool') {
				const v = (key === 'brand' ? item.brand : item.attrs?.[key]) as number;
				return vals.has(v > 0 ? '1' : '0');
			} else {
				const v = key === 'brand' ? item.brand : item.attrs?.[key];
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
	tags: Tag[]
): Item[] {
	if (!sortKey) return items;
	const key = sortKey;
	const dir = sortDir;
	return [...items].sort((a, b) => {
		let va: unknown, vb: unknown;
		if (key === 'name') {
			va = a.name;
			vb = b.name;
		} else if (key === 'tag') {
			const ta = a.tag_id ? tags.find(t => t.id === a.tag_id) : null;
			const tb = b.tag_id ? tags.find(t => t.id === b.tag_id) : null;
			va = ta?.name ?? '';
			vb = tb?.name ?? '';
		} else if (key === 'brand') {
			va = a.brand;
			vb = b.brand;
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

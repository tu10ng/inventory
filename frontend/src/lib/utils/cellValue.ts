import type { Item } from '$lib/types';
import type { ItemColumnDef } from '$lib/utils/columns';

export function getCellValue(item: Item, col: ItemColumnDef): unknown {
	// Core fields stay on item directly
	if (col.key === 'brand') return item.brand;
	if (col.key === 'default_qty') return item.default_qty;
	// Everything else lives in attrs
	return item.attrs?.[col.key] ?? null;
}

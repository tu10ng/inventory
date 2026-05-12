import type { Item } from '$lib/types';
import type { ItemColumnDef } from '$lib/utils/columns';

export function getCellValue(item: Item, col: ItemColumnDef): unknown {
	// All values live in attrs JSON
	return item.attrs?.[col.key] ?? null;
}

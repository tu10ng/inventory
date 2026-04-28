import type { TripItemEnriched } from '$lib/types';

export interface DragData {
	itemId: number;
	tagId: number | null;
}

let draggingItem = $state<DragData | null>(null);
let validSlotIds = $state<Set<number>>(new Set());

export function getDragState() {
	return {
		get draggingItem() {
			return draggingItem;
		},
		get validSlotIds() {
			return validSlotIds;
		}
	};
}

export function startDrag(item: DragData, enrichedItems: TripItemEnriched[]) {
	draggingItem = item;
	const slots = new Set<number>();
	for (const ti of enrichedItems) {
		if (ti.slot_id == null) continue;
		for (const c of ti.candidates) {
			if (c.id === item.itemId) {
				slots.add(ti.slot_id);
				break;
			}
		}
	}
	validSlotIds = slots;
}

export function endDrag() {
	draggingItem = null;
	validSlotIds = new Set();
}

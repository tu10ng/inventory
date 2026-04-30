import type { Trip, TripItemEnriched, Item, Category, Person, Tip } from '$lib/types';
import { STATUS_LABELS, TRIP_STATUS_LABELS } from './status';

interface GroupedItems {
	category: Category;
	items: TripItemEnriched[];
}

export function generateTripText(
	trip: Trip,
	groupedItems: GroupedItems[],
	allItems: Item[],
	people: Person[],
	tips: Tip[],
	totalChecked: number,
	totalItems: number
): string {
	const lines: string[] = [];

	lines.push(`行程: ${trip.name}`);
	if (trip.start_date || trip.end_date) {
		lines.push(`日期: ${trip.start_date || '?'} ~ ${trip.end_date || '?'}`);
	}
	lines.push(`状态: ${TRIP_STATUS_LABELS[trip.status] || trip.status}`);

	if (tips.length > 0) {
		lines.push('');
		lines.push('⚠️ 注意事项');
		for (const tip of tips) {
			lines.push(`- ${tip.content}`);
		}
	}

	lines.push('');
	lines.push(`📋 清单 (${totalChecked}/${totalItems})`);

	const itemMap = new Map<number, Item>();
	for (const item of allItems) {
		itemMap.set(item.id, item);
	}

	const personMap = new Map<number, string>();
	for (const p of people) {
		personMap.set(p.id, p.name);
	}

	for (const group of groupedItems) {
		const checked = group.items.filter((t) => t.checked).length;
		lines.push('');
		lines.push(`${group.category.icon} ${group.category.name} (${checked}/${group.items.length})`);

		for (const ti of group.items) {
			const parts: string[] = [];

			// Check mark
			parts.push(ti.checked ? '☑' : '☐');

			// Essential marker
			if (ti.is_essential) {
				parts.push('★');
			}

			// Item name: slot_name > item name > custom_name
			let name = '';
			if (ti.slot && ti.slot.slot_name) {
				name = ti.slot.slot_name;
			} else if (ti.item_id) {
				const item = itemMap.get(ti.item_id);
				name = item?.name || ti.custom_name || '未知物品';
			} else {
				name = ti.custom_name || '未知物品';
			}
			parts.push(name);

			// Quantity
			if (ti.qty > 1) {
				parts.push(`×${ti.qty}`);
			}

			// Person
			if (ti.person_id) {
				const personName = personMap.get(ti.person_id);
				if (personName) {
					parts.push(`(${personName})`);
				}
			}

			// Status
			if (ti.item_status && STATUS_LABELS[ti.item_status]) {
				parts.push(`[${STATUS_LABELS[ti.item_status]}]`);
			}

			// Notes
			if (ti.notes) {
				parts.push(`// ${ti.notes}`);
			}

			lines.push(`  ${parts.join(' ')}`);
		}
	}

	return lines.join('\n');
}

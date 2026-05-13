import { describe, it, expect } from 'vitest';
import { generateTripText } from './export';
import type { Trip, TripItemEnriched, Item, Category, Person, Tip } from '$lib/types';

describe('generateTripText', () => {
	const trip: Trip = {
		id: 1,
		name: '周末徒步',
		activity_id: 1,
		start_date: '2026-05-16',
		end_date: '2026-05-17',
		notes: '',
		status: 'planning'
	};

	const clothingCat: Category = { id: 1, name: '服装', icon: '👕', sort_order: 1 };
	const gearCat: Category = { id: 2, name: '装备', icon: '🎒', sort_order: 2 };

	const tripItems: TripItemEnriched[] = [
		{
			id: 1, trip_id: 1, item_id: 1, custom_name: '',
			person_id: null, qty: 1, checked: true, item_status: '',
			notes: '', sort_order: 0, is_essential: true, slot_id: null,
			slot: { id: 1, slot_name: '冲锋衣', category_id: 1, is_essential: true },
			candidates: []
		},
		{
			id: 2, trip_id: 1, item_id: 2, custom_name: '',
			person_id: 1, qty: 2, checked: false, item_status: 'need_buy',
			notes: '记得充电', sort_order: 1, is_essential: false, slot_id: null,
			slot: null,
			candidates: []
		}
	];

	const allItems: Item[] = [
		{ id: 1, category_id: 1, tag_id: null, attrs: { name: '冲锋衣', brand: '始祖鸟' } },
		{ id: 2, category_id: 2, tag_id: null, attrs: { name: '头灯', brand: 'Petzl' } }
	];

	const people: Person[] = [
		{ id: 1, name: '张三' }
	];

	const tips: Tip[] = [
		{ id: 1, activity_id: 1, content: '带足水', sort_order: 1 }
	];

	const groupedItems = [
		{ category: clothingCat, items: [tripItems[0]] },
		{ category: gearCat, items: [tripItems[1]] }
	];

	it('includes trip name', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('周末徒步');
	});

	it('includes date range', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('2026-05-16');
		expect(text).toContain('2026-05-17');
	});

	it('includes trip status', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		// Falls back to raw value since status defs aren't loaded in test
		expect(text).toContain('planning');
	});

	it('includes tips', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, tips, 1, 2);
		expect(text).toContain('带足水');
	});

	it('includes check count', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('(1/2)');
	});

	it('includes item name from slot', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('冲锋衣');
	});

	it('includes item name from allItems when no slot', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('头灯');
	});

	it('includes quantity when > 1', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('×2');
	});

	it('includes person name', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('张三');
	});

	it('includes checked marker', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('☑');
		expect(text).toContain('☐');
	});

	it('includes essential marker', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('★');
	});

	it('includes notes', () => {
		const text = generateTripText(trip, groupedItems, allItems, people, [], 1, 2);
		expect(text).toContain('记得充电');
	});
});

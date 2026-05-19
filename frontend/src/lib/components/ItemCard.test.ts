import { describe, it, expect, vi, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/svelte';

// Mock the dragState store before importing the component
vi.mock('$lib/stores/dragState.svelte', () => ({
	startDrag: vi.fn(),
	endDrag: vi.fn()
}));

import ItemCard from './ItemCard.svelte';
import type { Item } from '$lib/types';

afterEach(() => {
	cleanup();
});

const sampleItem: Item = {
	id: 1,
	type_id: null,
	attrs: { name: '冲锋衣', brand: '始祖鸟', model: 'Beta LT', default_qty: 2, warmth_rating: 30 }
};

describe('ItemCard', () => {
	it('renders item name', () => {
		render(ItemCard, { item: sampleItem });
		expect(screen.getByText('冲锋衣')).toBeTruthy();
	});

	it('renders brand and model in card-detail', () => {
		render(ItemCard, { item: sampleItem });
		// Brand and model render together as "始祖鸟 Beta LT"
		expect(screen.getByText('始祖鸟 Beta LT')).toBeTruthy();
	});

	it('renders qty badge as xN', () => {
		render(ItemCard, { item: sampleItem });
		expect(screen.getByText('x2')).toBeTruthy();
	});

	it('shows already-added class when alreadyAdded=true', () => {
		const { container } = render(ItemCard, {
			item: sampleItem,
			alreadyAdded: true
		});
		const card = container.querySelector('.item-card');
		expect(card?.classList.contains('already-added')).toBe(true);
	});

	it('calls onclick when clicked', async () => {
		const onclick = vi.fn();
		render(ItemCard, { item: sampleItem, onclick });
		const card = document.querySelector('.item-card');
		card?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
		expect(onclick).toHaveBeenCalled();
	});
});

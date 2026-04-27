<script lang="ts">
	import type { Item, Category } from '$lib/types';
	import SearchFilter from './SearchFilter.svelte';
	import ItemCard from './ItemCard.svelte';

	let { items, categories, tripItemIds, onAddItem }: {
		items: Item[];
		categories: Category[];
		tripItemIds: Set<number>;
		onAddItem: (itemId: number) => void;
	} = $props();

	let search = $state('');
	let filterCategoryId = $state<number | null>(null);

	const filteredItems = $derived.by(() => {
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
		return list;
	});

	const groupedFiltered = $derived.by(() => {
		const groups: { category: Category; items: Item[] }[] = [];
		const catMap = new Map<number, Item[]>();

		for (const item of filteredItems) {
			if (!catMap.has(item.category_id)) catMap.set(item.category_id, []);
			catMap.get(item.category_id)!.push(item);
		}

		for (const cat of categories) {
			const catItems = catMap.get(cat.id);
			if (catItems && catItems.length > 0) {
				groups.push({ category: cat, items: catItems });
			}
		}
		return groups;
	});

	function getCategoryIcon(catId: number): string {
		return categories.find((c) => c.id === catId)?.icon ?? '📦';
	}
</script>

<div class="inventory-panel">
	<div class="panel-header">
		<h3>物品库</h3>
		<span class="item-count">{filteredItems.length} 件</span>
	</div>

	<SearchFilter
		{search}
		categoryId={filterCategoryId}
		{categories}
		onSearchChange={(v) => (search = v)}
		onCategoryChange={(id) => (filterCategoryId = id)}
		dark
	/>

	<div class="inventory-grid-container">
		{#each groupedFiltered as group}
			<div class="group-label">{group.category.icon} {group.category.name}</div>
			<div class="inventory-grid">
				{#each group.items as item (item.id)}
					<ItemCard
						name={item.name}
						brand={item.brand}
						model={item.model}
						categoryIcon={getCategoryIcon(item.category_id)}
						qty={item.default_qty}
						alreadyAdded={tripItemIds.has(item.id)}
						onclick={() => onAddItem(item.id)}
					/>
				{/each}
			</div>
		{/each}

		{#if filteredItems.length === 0}
			<div class="empty">没有匹配的物品</div>
		{/if}
	</div>
</div>

<style>
	.inventory-panel {
		background: var(--inventory-bg);
		border-radius: 12px;
		padding: 16px;
		color: var(--inventory-text);
	}
	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 12px;
	}
	.panel-header h3 {
		margin: 0;
		font-size: 16px;
	}
	.item-count {
		font-size: 13px;
		color: var(--inventory-text-secondary);
	}
	.group-label {
		font-size: 13px;
		color: var(--inventory-text-secondary);
		margin: 12px 0 6px;
		font-weight: 500;
	}
	.group-label:first-child {
		margin-top: 0;
	}
	.inventory-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
		gap: 8px;
	}
	.inventory-grid-container {
		overflow-y: auto;
	}
	.empty {
		text-align: center;
		color: var(--inventory-text-secondary);
		padding: 40px 0;
	}
</style>

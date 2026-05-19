<script lang="ts">
	import type { Item, Type, TripItemEnriched } from '$lib/types';
	import { getRootTypeId, getRootTypeName } from '$lib/utils/columns';
	import SearchFilter from './SearchFilter.svelte';
	import ItemCard from './ItemCard.svelte';

	let { items, types, tripItemIds, enrichedItems, onHoverItem = null }: {
		items: Item[];
		types: Type[];
		tripItemIds: Set<number>;
		enrichedItems: TripItemEnriched[];
		onHoverItem?: ((itemId: number | null) => void) | null;
	} = $props();

	let search = $state('');
	let filterRootTypeId = $state<number | null>(null);

	const rootTypes = $derived(types.filter(t => t.parent_id === null).sort((a, b) => a.sort_order - b.sort_order));

	const filteredItems = $derived.by(() => {
		let list = items;
		if (search) {
			const q = search.toLowerCase();
			list = list.filter(
				(i) =>
					String(i.attrs?.name ?? '').toLowerCase().includes(q) ||
					String(i.attrs?.brand ?? '').toLowerCase().includes(q) ||
					String(i.attrs?.model ?? '').toLowerCase().includes(q)
			);
		}
		if (filterRootTypeId !== null) {
			list = list.filter((i) => getRootTypeId(i.type_id, types) === filterRootTypeId);
		}
		return list;
	});

	const groupedFiltered = $derived.by(() => {
		const groups: { rootType: Type; items: Item[] }[] = [];
		const typeMap = new Map<number, Item[]>();

		for (const item of filteredItems) {
			const rootId = getRootTypeId(item.type_id, types);
			if (rootId == null) continue;
			if (!typeMap.has(rootId)) typeMap.set(rootId, []);
			typeMap.get(rootId)!.push(item);
		}

		for (const rt of rootTypes) {
			const rtItems = typeMap.get(rt.id);
			if (rtItems && rtItems.length > 0) {
				groups.push({ rootType: rt, items: rtItems });
			}
		}
		return groups;
	});
</script>

<div class="inventory-panel">
	<div class="panel-header">
		<h3>物品库</h3>
		<span class="item-count">{filteredItems.length} 件</span>
	</div>

	<SearchFilter
		{search}
		rootTypeId={filterRootTypeId}
		{rootTypes}
		onSearchChange={(v) => (search = v)}
		onRootTypeChange={(id) => (filterRootTypeId = id)}
	/>

	<p class="drag-hint">拖拽物品卡片到左侧清单</p>

	<div class="inventory-grid-container">
		{#each groupedFiltered as group}
			<div class="group-label">{group.rootType.name}</div>
			<div class="inventory-grid">
				{#each group.items as item (item.id)}
					<ItemCard
						{item}
						alreadyAdded={tripItemIds.has(item.id)}
						itemId={item.id}
						{enrichedItems}
						onHover={onHoverItem}
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
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 16px;
		color: var(--text);
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
		color: var(--text-secondary);
	}
	.drag-hint {
		font-size: 12px;
		color: var(--text-secondary);
		margin: 0 0 12px;
	}
	.group-label {
		font-size: 13px;
		color: var(--text-secondary);
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
		color: var(--text-secondary);
		padding: 40px 0;
	}
</style>

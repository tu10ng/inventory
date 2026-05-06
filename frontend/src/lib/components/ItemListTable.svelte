<script lang="ts">
	import type { Item, Category, Tag } from '$lib/types';

	let {
		items,
		categories,
		tags,
		selectedItemId,
		collapsedCategories,
		onSelect,
		onToggleCategory,
	}: {
		items: Item[];
		categories: Category[];
		tags: Tag[];
		selectedItemId: number | null;
		collapsedCategories: Set<number>;
		onSelect: (item: Item) => void;
		onToggleCategory: (catId: number) => void;
	} = $props();

	const groupedItems = $derived.by(() => {
		const groups: { category: Category; items: Item[] }[] = [];
		for (const cat of categories) {
			const catItems = items.filter((i) => i.category_id === cat.id);
			if (catItems.length > 0) {
				groups.push({ category: cat, items: catItems });
			}
		}
		return groups;
	});

	function getTag(item: Item): Tag | undefined {
		if (!item.tag_id) return undefined;
		return tags.find(t => t.id === item.tag_id);
	}
</script>

<div class="item-list-table">
	{#each groupedItems as group (group.category.id)}
		<button
			class="category-row"
			onclick={() => onToggleCategory(group.category.id)}
		>
			<span class="collapse-icon">{collapsedCategories.has(group.category.id) ? '▶' : '▼'}</span>
			<span class="cat-icon">{group.category.icon}</span>
			<span class="cat-name">{group.category.name}</span>
			<span class="cat-count">({group.items.length})</span>
		</button>

		{#if !collapsedCategories.has(group.category.id)}
			{#each group.items as item (item.id)}
				{@const itemTag = getTag(item)}
				<button
					class="item-row"
					class:selected={item.id === selectedItemId}
					onclick={() => onSelect(item)}
				>
					<span class="item-name">{item.name}</span>
					{#if itemTag}
						<span class="item-tag">{itemTag.name}</span>
					{/if}
					{#if item.brand || item.model}
						<span class="item-brand">{item.brand}{item.brand && item.model ? ' ' : ''}{item.model}</span>
					{/if}
				</button>
			{/each}
		{/if}
	{/each}

	{#if items.length === 0}
		<div class="empty-state">暂无物品</div>
	{:else if groupedItems.length === 0}
		<div class="empty-state">没有匹配的物品</div>
	{/if}
</div>

<style>
	.item-list-table {
		border: 1px solid var(--border);
		border-radius: 8px;
		overflow: hidden;
		background: var(--surface);
	}

	.category-row {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 8px 12px;
		background: var(--bg);
		border: none;
		border-bottom: 1px solid var(--border);
		cursor: pointer;
		font-size: 13px;
		font-weight: 600;
		color: var(--text);
		text-align: left;
	}
	.category-row:hover {
		background: color-mix(in srgb, var(--bg), var(--primary) 5%);
	}

	.collapse-icon {
		font-size: 10px;
		width: 14px;
		color: var(--text-secondary);
	}
	.cat-icon {
		font-size: 15px;
	}
	.cat-name {
		flex: 1;
	}
	.cat-count {
		color: var(--text-secondary);
		font-weight: 400;
		font-size: 12px;
	}

	.item-row {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 12px 6px 36px;
		border: none;
		border-bottom: 1px solid color-mix(in srgb, var(--border), transparent 50%);
		background: var(--surface);
		cursor: pointer;
		font-size: 13px;
		color: var(--text);
		text-align: left;
		transition: background 0.1s;
	}
	.item-row:hover {
		background: color-mix(in srgb, var(--surface), var(--primary) 6%);
	}
	.item-row.selected {
		background: color-mix(in srgb, var(--surface), var(--primary) 12%);
		box-shadow: inset 3px 0 0 var(--primary);
	}
	.item-row:last-child {
		border-bottom: none;
	}

	.item-name {
		font-weight: 500;
		flex-shrink: 0;
	}
	.item-tag {
		font-size: 11px;
		background: #eef2ff;
		color: var(--primary);
		padding: 0 6px;
		border-radius: 8px;
		border: 1px solid #c7d2fe;
		flex-shrink: 0;
	}
	.item-brand {
		font-size: 12px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.empty-state {
		padding: 40px;
		text-align: center;
		color: var(--text-secondary);
	}
</style>

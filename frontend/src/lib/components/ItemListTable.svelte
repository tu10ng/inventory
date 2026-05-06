<script lang="ts">
	import type { Item, Category, Tag, ItemColumnDef } from '$lib/types';

	let {
		items,
		categories,
		tags,
		usageStats,
		visibleColumns,
		selectedItemId,
		collapsedCategories,
		onSelect,
		onToggleCategory,
	}: {
		items: Item[];
		categories: Category[];
		tags: Tag[];
		usageStats: Map<number, number>;
		visibleColumns: ItemColumnDef[];
		selectedItemId: number | null;
		collapsedCategories: Set<number>;
		onSelect: (item: Item) => void;
		onToggleCategory: (catId: number) => void;
	} = $props();

	const ctx = $derived({ tags, usageStats });

	const gridTemplateColumns = $derived(
		visibleColumns.map((c) => c.width).join(' ')
	);

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

	function renderValue(col: ItemColumnDef, item: Item): string {
		const val = col.getValue(item, ctx);
		if (val === null || val === undefined || val === '' || val === 0) return '';
		switch (col.render) {
			case 'bool':
				return val ? '✓' : '';
			case 'weight':
				return val ? `${val}g` : '';
			case 'stars': {
				const n = Number(val);
				return '★'.repeat(n) + '☆'.repeat(5 - n);
			}
			default:
				return String(val);
		}
	}

	function barPercent(col: ItemColumnDef, item: Item): number {
		const val = Number(col.getValue(item, ctx));
		if (col.key === 'warmth') return Math.min(100, Math.round((val / 50) * 100));
		return 0;
	}
</script>

<div class="item-list-table">
	<!-- Header -->
	<div class="table-header" style:grid-template-columns={gridTemplateColumns}>
		{#each visibleColumns as col (col.key)}
			<div class="th">{col.label}</div>
		{/each}
	</div>

	<!-- Groups -->
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
				<button
					class="item-row"
					class:selected={item.id === selectedItemId}
					style:grid-template-columns={gridTemplateColumns}
					onclick={() => onSelect(item)}
				>
					{#each visibleColumns as col (col.key)}
						<div class="td" class:td-name={col.key === 'name'} class:td-bool={col.render === 'bool'} class:td-stars={col.render === 'stars'} class:td-tag={col.render === 'tag'}>
							{#if col.render === 'bar'}
								{@const pct = barPercent(col, item)}
								{#if pct > 0}
									<div class="bar-wrap">
										<div class="bar-fill" style:width="{pct}%"></div>
									</div>
									<span class="bar-val">{col.getValue(item, ctx)}</span>
								{/if}
							{:else}
								{renderValue(col, item)}
							{/if}
						</div>
					{/each}
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

	.table-header {
		display: grid;
		gap: 0;
		padding: 6px 12px;
		background: var(--bg);
		border-bottom: 1px solid var(--border);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	.th {
		padding: 2px 4px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
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
		display: grid;
		gap: 0;
		width: 100%;
		padding: 5px 12px;
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

	.td {
		padding: 2px 4px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: flex;
		align-items: center;
	}
	.td-name {
		font-weight: 500;
	}
	.td-bool {
		color: var(--success);
		font-size: 14px;
	}
	.td-stars {
		color: #f0ad4e;
		font-size: 11px;
		letter-spacing: 0.5px;
	}
	.td-tag {
		font-size: 11px;
		color: var(--primary);
	}

	.bar-wrap {
		flex: 1;
		height: 6px;
		background: var(--bg);
		border-radius: 3px;
		overflow: hidden;
	}
	.bar-fill {
		height: 100%;
		background: var(--primary);
		border-radius: 3px;
	}
	.bar-val {
		font-size: 11px;
		color: var(--text-secondary);
		margin-left: 6px;
		width: 24px;
		text-align: right;
		flex-shrink: 0;
	}

	.empty-state {
		padding: 40px;
		text-align: center;
		color: var(--text-secondary);
	}
</style>

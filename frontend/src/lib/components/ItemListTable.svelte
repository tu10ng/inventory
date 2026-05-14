<script lang="ts">
	import type { Item, Category, Tag } from '$lib/types';
	import { itemName, itemBrand, itemModel } from '$lib/types';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import type { ItemGroup } from '$lib/utils/itemFilters';
	import { getCellValue } from '$lib/utils/cellValue';
	import CellRenderer from './CellRenderer.svelte';
	import ItemGroupBlock from './ItemGroupBlock.svelte';

	let {
		items,
		categories,
		tags,
		visibleColumns = [],
		selectedItemId,
		collapsedCategories,
		sortKey = null,
		sortDir = 'asc',
		columnFilters = new Map(),
		groupBy = null,
		groupedData = null,
		selectable = true,
		selectedIds = new Set(),
		onSelect,
		onToggleCategory,
		onSort,
		onFilterChange,
		onToggleSelect,
	}: {
		items: Item[];
		categories: Category[];
		tags: Tag[];
		visibleColumns?: ItemColumnDef[];
		selectedItemId: number | null;
		collapsedCategories: Set<number>;
		sortKey?: string | null;
		sortDir?: 'asc' | 'desc';
		columnFilters?: Map<string, Set<string>>;
		groupBy?: { key: string; label: string } | null;
		groupedData?: Map<number, { groups: ItemGroup[]; ungrouped: Item[] }> | null;
		selectable?: boolean;
		selectedIds?: Set<number>;
		onSelect: (item: Item) => void;
		onToggleCategory: (catId: number) => void;
		onSort?: (key: string) => void;
		onFilterChange?: (key: string, values: Set<string>) => void;
		onToggleSelect?: (id: number) => void;
	} = $props();

	let openFilter = $state<string | null>(null);

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

	const tagMap = $derived(new Map(tags.map(t => [t.id, t])));

	function getTag(item: Item): Tag | undefined {
		if (!item.tag_id) return undefined;
		return tagMap.get(item.tag_id);
	}

	function handleSort(key: string) {
		onSort?.(key);
	}

	function toggleFilter(key: string, e: MouseEvent) {
		e.stopPropagation();
		openFilter = openFilter === key ? null : key;
	}

	function handleFilterValue(colKey: string, value: string) {
		const current = columnFilters.get(colKey) ?? new Set<string>();
		const next = new Set(current);
		if (next.has(value)) next.delete(value);
		else next.add(value);
		onFilterChange?.(colKey, next);
	}

	function clearFilter(colKey: string, e: MouseEvent) {
		e.stopPropagation();
		onFilterChange?.(colKey, new Set());
		openFilter = null;
	}

	function getUniqueValues(col: ItemColumnDef): string[] {
		const vals = new Set<string>();
		for (const item of items) {
			if (col.type === 'tag') {
				const t = getTag(item);
				vals.add(t ? t.name : '-');
			} else if (col.type === 'bool') {
				const v = getCellValue(item, col) as number;
				vals.add(v > 0 ? '1' : '0');
			} else {
				const v = getCellValue(item, col);
				vals.add(v ? String(v) : '-');
			}
		}
		return [...vals].sort();
	}

	function getFilterDisplayLabel(col: ItemColumnDef, val: string): string {
		if (col.type === 'bool') return val === '1' ? '✓' : '✗';
		return val;
	}

	function closeFilter(e: MouseEvent) {
		if (openFilter) openFilter = null;
	}
</script>

<svelte:window onclick={closeFilter} />

<div class="item-list-table" style="--extra-cols: {visibleColumns.length}">
	<!-- Header -->
	{#if visibleColumns.length > 0}
		<div class="header-row">
			<span class="check-col"><input type="checkbox" checked={selectedIds.size === items.length && items.length > 0} onchange={() => { if (onToggleSelect) { if (selectedIds.size === items.length) { items.forEach(i => onToggleSelect(i.id)); } else { items.forEach(i => { if (!selectedIds.has(i.id)) onToggleSelect(i.id); }); } } }} /></span>
			<button class="hdr-name hdr-btn" onclick={() => handleSort('name')}>
				<span>名称</span>
				{#if sortKey === 'name'}
					<span class="sort-icon">{sortDir === 'asc' ? '▲' : '▼'}</span>
				{/if}
			</button>
			{#each visibleColumns as col (col.key)}
				<span class="hdr-col">
					<button class="hdr-btn" onclick={() => handleSort(col.key)}>
						<span>{col.label}</span>
						{#if sortKey === col.key}
							<span class="sort-icon">{sortDir === 'asc' ? '▲' : '▼'}</span>
						{/if}
					</button>
					{#if col.filterable !== false && (col.type === 'text' || col.type === 'tag' || col.type === 'bool')}
						<button
							class="filter-btn"
							class:active={columnFilters.has(col.key) && columnFilters.get(col.key)!.size > 0}
							onclick={(e) => toggleFilter(col.key, e)}
							title="筛选"
						>▿</button>
						{#if openFilter === col.key}
							<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
						<div class="filter-dropdown" role="region" aria-label="筛选选项" onclick={(e) => e.stopPropagation()}>
								<div class="filter-header">
									<span>筛选: {col.label}</span>
									<button class="filter-clear" onclick={(e) => clearFilter(col.key, e)}>清除</button>
								</div>
								<div class="filter-options">
									{#each getUniqueValues(col) as val}
										{@const checked = columnFilters.get(col.key)?.has(val) ?? false}
										<label class="filter-option">
											<input type="checkbox" checked={checked} onchange={() => handleFilterValue(col.key, val)} />
											<span>{getFilterDisplayLabel(col, val)}</span>
										</label>
									{/each}
								</div>
							</div>
						{/if}
					{/if}
				</span>
			{/each}
		</div>
	{:else}
		<div class="header-row">
			<span class="check-col"><input type="checkbox" checked={selectedIds.size === items.length && items.length > 0} onchange={() => { if (onToggleSelect) { if (selectedIds.size === items.length) { items.forEach(i => onToggleSelect(i.id)); } else { items.forEach(i => { if (!selectedIds.has(i.id)) onToggleSelect(i.id); }); } } }} /></span>
			<button class="hdr-name hdr-btn" onclick={() => handleSort('name')}>
				<span>名称</span>
				{#if sortKey === 'name'}
					<span class="sort-icon">{sortDir === 'asc' ? '▲' : '▼'}</span>
				{/if}
			</button>
		</div>
	{/if}

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
			{#if groupBy && groupedData}
				{@const catData = groupedData.get(group.category.id)}
				{#if catData}
					{#each catData.groups as grp (grp.value)}
						<ItemGroupBlock
							label={grp.label}
							value={grp.value}
							items={grp.items}
							{visibleColumns}
							{selectedItemId}
							{tags}
							{selectedIds}
							{onSelect}
							{onToggleSelect}
						/>
					{/each}
					{#if catData.ungrouped.length > 0}
						<div class="ungrouped-header">
							<span class="check-col"></span>
							<span>··· 未分组 ···</span>
						</div>
						{#each catData.ungrouped as item (item.id)}
							<button
								class="item-row"
								class:selected={item.id === selectedItemId}
								onclick={() => onSelect(item)}
							>
								<span class="check-col"><input type="checkbox" checked={selectedIds.has(item.id)} onclick={(e) => e.stopPropagation()} onchange={() => onToggleSelect?.(item.id)} /></span>
								<span class="item-name">{itemName(item)}</span>
								{#each visibleColumns as col (col.key)}
									{@const v = getCellValue(item, col)}
									<span class="cell cell-{col.type}">
										{#if v != null && String(v) !== ''}
											<span class="cell-text">{v}{col.suffix ?? ''}</span>
										{:else}
											<span class="cell-empty">-</span>
										{/if}
									</span>
								{/each}
							</button>
						{/each}
					{/if}
				{:else}
					{#each group.items as item (item.id)}
						<button
							class="item-row"
							class:selected={item.id === selectedItemId}
							onclick={() => onSelect(item)}
						>
							<span class="check-col"><input type="checkbox" checked={selectedIds.has(item.id)} onclick={(e) => e.stopPropagation()} onchange={() => onToggleSelect?.(item.id)} /></span>
							<span class="item-name">{itemName(item)}</span>
							{#each visibleColumns as col (col.key)}
								<CellRenderer {item} {col} tag={getTag(item)} />
							{/each}
						</button>
					{/each}
				{/if}
			{:else}
				{#each group.items as item (item.id)}
					{@const itemTag = getTag(item)}
					<button
						class="item-row"
						class:selected={item.id === selectedItemId}
						onclick={() => onSelect(item)}
					>
						<span class="check-col"><input type="checkbox" checked={selectedIds.has(item.id)} onclick={(e) => e.stopPropagation()} onchange={() => onToggleSelect?.(item.id)} /></span>
						<span class="item-name">
							{itemName(item)}
							{#if visibleColumns.length === 0 && itemTag}
								<span class="inline-tag">{itemTag.name}</span>
							{/if}
							{#if visibleColumns.length === 0 && (String(item.attrs?.brand ?? '') || String(item.attrs?.model ?? ''))}
								<span class="inline-brand">{itemBrand(item)}{itemBrand(item) && itemModel(item) ? ' ' : ''}{itemModel(item)}</span>
							{/if}
						</span>
						{#each visibleColumns as col (col.key)}
							<CellRenderer {item} {col} tag={getTag(item)} />
						{/each}
					</button>
				{/each}
			{/if}
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
		overflow-y: auto;
		overflow-x: hidden;
		background: var(--surface);
		flex: 1;
		min-height: 0;
	}

	.header-row {
		display: grid;
		grid-template-columns: 32px 1fr repeat(var(--extra-cols, 0), auto);
		gap: 0;
		padding: 4px 12px 4px 8px;
		background: var(--bg);
		border-bottom: 1px solid var(--border);
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.3px;
		position: sticky;
		top: 0;
		z-index: 5;
	}
	.hdr-col {
		text-align: center;
		min-width: 56px;
		padding: 0 6px;
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 2px;
	}

	.hdr-btn {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		background: none;
		border: none;
		color: inherit;
		font: inherit;
		text-transform: inherit;
		letter-spacing: inherit;
		cursor: pointer;
		padding: 2px 4px;
		border-radius: 3px;
		white-space: nowrap;
	}
	.hdr-btn:hover {
		background: color-mix(in srgb, var(--border), transparent 30%);
	}
	.hdr-name {
		text-align: left;
	}

	.sort-icon {
		font-size: 8px;
		opacity: 0.8;
	}

	.filter-btn {
		background: none;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: 10px;
		padding: 1px 3px;
		border-radius: 3px;
		opacity: 0.5;
		line-height: 1;
	}
	.filter-btn:hover {
		opacity: 1;
		background: color-mix(in srgb, var(--border), transparent 30%);
	}
	.filter-btn.active {
		opacity: 1;
		color: var(--primary);
	}

	.filter-dropdown {
		position: absolute;
		top: 100%;
		right: 0;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 6px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		min-width: 140px;
		max-height: 240px;
		z-index: 20;
		text-transform: none;
		letter-spacing: normal;
		font-weight: 400;
	}
	.filter-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 6px 10px;
		border-bottom: 1px solid var(--border);
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
	}
	.filter-clear {
		background: none;
		border: none;
		color: var(--primary);
		cursor: pointer;
		font-size: 11px;
		padding: 0;
	}
	.filter-clear:hover {
		text-decoration: underline;
	}
	.filter-options {
		padding: 4px 0;
		max-height: 190px;
		overflow-y: auto;
	}
	.filter-option {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		font-size: 12px;
		color: var(--text);
		cursor: pointer;
	}
	.filter-option:hover {
		background: color-mix(in srgb, var(--surface), var(--primary) 6%);
	}
	.filter-option input[type='checkbox'] {
		width: 14px;
		height: 14px;
		accent-color: var(--primary);
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
		grid-template-columns: 32px 1fr repeat(var(--extra-cols, 0), auto);
		align-items: center;
		gap: 0;
		width: 100%;
		padding: 6px 12px 6px 8px;
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
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.inline-tag {
		font-size: 11px;
		background: #eef2ff;
		color: var(--primary);
		padding: 0 6px;
		border-radius: 8px;
		border: 1px solid #c7d2fe;
		flex-shrink: 0;
	}
	.inline-brand {
		font-size: 12px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}

	.cell {
		text-align: center;
		min-width: 56px;
		padding: 0 6px;
		font-size: 12px;
	}
	.cell-pill {
		font-size: 11px;
		background: #eef2ff;
		color: var(--primary);
		padding: 0 6px;
		border-radius: 8px;
		border: 1px solid #c7d2fe;
		white-space: nowrap;
	}
	.cell-text {
		color: var(--text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 100px;
		display: inline-block;
	}
	.cell-num {
		font-variant-numeric: tabular-nums;
		color: var(--text);
	}
	.cell-empty {
		color: var(--text-secondary);
		opacity: 0.4;
	}
	.cell-bool.yes {
		color: var(--success);
		font-weight: 600;
	}
	.cell-bool.no {
		color: var(--text-secondary);
		opacity: 0.3;
	}
	.cell-bar {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}
	.bar-track {
		display: inline-block;
		width: 36px;
		height: 6px;
		background: var(--border);
		border-radius: 3px;
		overflow: hidden;
	}
	.bar-fill {
		display: block;
		height: 100%;
		background: var(--primary);
		border-radius: 3px;
	}
	.bar-val {
		font-size: 10px;
		color: var(--text-secondary);
		min-width: 14px;
		text-align: right;
	}
	.cell-stars {
		font-size: 11px;
		color: #f59e0b;
		letter-spacing: -1px;
		white-space: nowrap;
	}

	.ungrouped-header {
		display: grid;
		grid-template-columns: 32px 1fr;
		padding: 6px 12px 6px 8px;
		font-size: 11px;
		color: var(--text-secondary);
		border-bottom: 1px dotted var(--border);
		opacity: 0.7;
	}

	.check-col {
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.check-col input[type='checkbox'] {
		width: 14px;
		height: 14px;
		accent-color: var(--primary);
		cursor: pointer;
	}

	.empty-state {
		padding: 40px;
		text-align: center;
		color: var(--text-secondary);
	}
</style>

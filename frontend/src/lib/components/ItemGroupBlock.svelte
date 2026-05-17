<script lang="ts">
	import type { Item } from '$lib/types';
	import { itemName, itemBrand, itemModel } from '$lib/types';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import { buildTypePath } from '$lib/utils/columns';
	import type { Type } from '$lib/types';
	import type { TypeTreeGroup } from '$lib/utils/itemFilters';
	import CellRenderer from './CellRenderer.svelte';
	import ItemGroupBlock from './ItemGroupBlock.svelte';

	let {
		label,
		value = '',
		items,
		visibleColumns = [],
		selectedItemId,
		types = [],
		selectedIds = new Set(),
		children = [] as TypeTreeGroup[],
		depth = 0,
		onSelect,
		onToggleSelect,
	}: {
		label: string;
		value?: string;
		items: Item[];
		visibleColumns?: ItemColumnDef[];
		selectedItemId: number | null;
		types?: Type[];
		selectedIds?: Set<number>;
		children?: TypeTreeGroup[];
		depth?: number;
		onSelect: (item: Item) => void;
		onToggleSelect?: (id: number) => void;
	} = $props();

	let collapsed = $state(false);

	const typeMap = $derived(new Map(types.map(t => [t.id, t])));

	function getType(item: Item): Type | undefined {
		if (!item.type_id) return undefined;
		return typeMap.get(item.type_id);
	}

	function getTypeDisplay(item: Item): string {
		const t = getType(item);
		return t ? buildTypePath(t.id, types) : '-';
	}
</script>

<div class="type-tree-block" style="margin-left: {depth * 1.2}em">
	<button class="tree-block-header" onclick={() => (collapsed = !collapsed)}>
		<span class="fold-arrow">{collapsed ? '▶' : '▼'}</span>
		<span class="tree-label">{label}</span>
		<span class="tree-count">({items.length})</span>
	</button>

	{#if !collapsed}
		<div class="tree-block-body">
			{#each items as item (item.id)}
				{@const itemType = getType(item)}
				<button
					class="item-row"
					class:selected={item.id === selectedItemId}
					onclick={() => onSelect(item)}
				>
					<span class="check-col"><input type="checkbox" checked={selectedIds.has(item.id)} onclick={(e) => e.stopPropagation()} onchange={() => onToggleSelect?.(item.id)} /></span>
					<span class="item-name">
						{itemName(item)}
						{#if visibleColumns.length === 0 && itemType}
							<span class="inline-type">{itemType.name}</span>
						{/if}
						{#if visibleColumns.length === 0 && (String(item.attrs?.brand ?? '') || String(item.attrs?.model ?? ''))}
							<span class="inline-brand">{itemBrand(item)}{itemBrand(item) && itemModel(item) ? ' ' : ''}{itemModel(item)}</span>
						{/if}
					</span>
					{#each visibleColumns as col (col.key)}
						<CellRenderer {item} {col} typeDisplay={getTypeDisplay(item)} />
					{/each}
				</button>
			{/each}

			{#each children as child}
				<ItemGroupBlock
					label={child.type.name}
					value={String(child.type.id)}
					items={child.items}
					children={child.children}
					depth={depth + 1}
					{visibleColumns}
					{selectedItemId}
					{types}
					{selectedIds}
					{onSelect}
					{onToggleSelect}
				/>
			{/each}
		</div>
	{/if}
</div>

<style>
	.type-tree-block {
		border: 1px solid var(--border);
		border-radius: 6px;
		margin: 4px 12px 4px 12px;
		overflow: hidden;
	}
	.tree-block-header {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 6px 10px;
		background: color-mix(in srgb, var(--surface), var(--primary) 6%);
		border: none;
		border-bottom: 1px solid var(--border);
		cursor: pointer;
		font-size: 12px;
		font-weight: 600;
		color: var(--text);
		text-align: left;
		transition: background 0.1s;
	}
	.tree-block-header:hover {
		background: color-mix(in srgb, var(--surface), var(--primary) 12%);
	}

	.fold-arrow {
		font-size: 9px;
		width: 12px;
		flex-shrink: 0;
		color: var(--text-secondary);
	}
	.tree-label {
		flex: 1;
	}
	.tree-count {
		font-size: 11px;
		font-weight: 400;
		color: var(--text-secondary);
		background: color-mix(in srgb, var(--surface), var(--border) 50%);
		padding: 0 6px;
		border-radius: 8px;
	}

	/* items render inside tree-block-body; children blocks add their own container */

	.item-row {
		display: grid;
		grid-template-columns: 32px 1fr repeat(var(--extra-cols, 0), auto);
		align-items: center;
		gap: 0;
		width: 100%;
		padding: 6px 12px 6px 8px;
		border: none;
		border-bottom: 1px solid color-mix(in srgb, var(--border), transparent 70%);
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
	.inline-type {
		font-size: 11px;
		background: #eef2ff;
		color: var(--primary);
		padding: 0 6px;
		border-radius: 8px;
		border: 1px solid #c7d2fe;
		flex-shrink: 0;
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

	.inline-brand {
		font-size: 12px;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
	}
</style>

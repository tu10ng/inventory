<script lang="ts">
	import type { Item } from '$lib/types';
	import { itemName, itemBrand, itemModel } from '$lib/types';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import { buildTypePath } from '$lib/utils/columns';
	import type { Type } from '$lib/types';
	import CellRenderer from './CellRenderer.svelte';

	let {
		label,
		value,
		items,
		visibleColumns = [],
		selectedItemId,
		types = [],
		selectedIds = new Set(),
		onSelect,
		onToggleSelect,
	}: {
		label: string;
		value: string;
		items: Item[];
		visibleColumns?: ItemColumnDef[];
		selectedItemId: number | null;
		types?: Type[];
		selectedIds?: Set<number>;
		onSelect: (item: Item) => void;
		onToggleSelect?: (id: number) => void;
	} = $props();

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

<fieldset class="group-block">
	<legend class="group-label">{label}</legend>
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
</fieldset>

<style>
	.group-block {
		border: 1px dashed var(--border);
		border-radius: 6px;
		padding: 8px 0 4px;
		margin: 6px 12px 6px 36px;
	}
	.group-label {
		font-size: 11px;
		font-weight: 600;
		background: var(--primary);
		color: white;
		padding: 2px 10px;
		border-radius: 10px;
		margin-left: 8px;
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

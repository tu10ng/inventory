<script lang="ts">
	import type { Item } from '$lib/types';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import type { Tag } from '$lib/types';
	import CellRenderer from './CellRenderer.svelte';

	let {
		label,
		value,
		items,
		visibleColumns = [],
		selectedItemId,
		tags = [],
		onSelect,
	}: {
		label: string;
		value: string;
		items: Item[];
		visibleColumns?: ItemColumnDef[];
		selectedItemId: number | null;
		tags?: Tag[];
		onSelect: (item: Item) => void;
	} = $props();

	const tagMap = $derived(new Map(tags.map(t => [t.id, t])));

	function getTag(item: Item): Tag | undefined {
		if (!item.tag_id) return undefined;
		return tagMap.get(item.tag_id);
	}
</script>

<fieldset class="group-block">
	<legend class="group-label">{label}</legend>
	{#each items as item (item.id)}
		{@const itemTag = getTag(item)}
		<button
			class="item-row"
			class:selected={item.id === selectedItemId}
			onclick={() => onSelect(item)}
		>
			<span class="item-name">
				{item.name}
				{#if visibleColumns.length === 0 && itemTag}
					<span class="inline-tag">{itemTag.name}</span>
				{/if}
				{#if visibleColumns.length === 0 && (item.brand || item.model)}
					<span class="inline-brand">{item.brand}{item.brand && item.model ? ' ' : ''}{item.model}</span>
				{/if}
			</span>
			{#each visibleColumns as col (col.key)}
				<CellRenderer {item} {col} tag={getTag(item)} />
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
		grid-template-columns: 1fr repeat(var(--extra-cols, 0), auto);
		align-items: center;
		gap: 0;
		width: 100%;
		padding: 6px 12px 6px 12px;
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
</style>

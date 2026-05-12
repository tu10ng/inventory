<script lang="ts">
	import type { Item, Tag } from '$lib/types';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import { getCellValue } from '$lib/utils/cellValue';

	let { item, col, tag }: {
		item: Item;
		col: ItemColumnDef;
		tag?: Tag | null;
	} = $props();
</script>

<span class="cell cell-{col.type}">
	{#if col.type === 'tag'}
		{#if tag}
			<span class="cell-pill">{tag.name}</span>
		{:else}
			<span class="cell-empty">-</span>
		{/if}
	{:else if col.type === 'text'}
		{@const v = getCellValue(item, col)}
		{#if v}
			<span class="cell-text">{v}</span>
		{:else}
			<span class="cell-empty">-</span>
		{/if}
	{:else if col.type === 'number'}
		{@const v = getCellValue(item, col) as number}
		{#if v}
			<span class="cell-num">{v}{col.suffix ? col.suffix : ''}</span>
		{:else}
			<span class="cell-empty">-</span>
		{/if}
	{:else if col.type === 'weight'}
		{@const v = getCellValue(item, col) as number}
		{#if v}
			<span class="cell-num">{v}g</span>
		{:else}
			<span class="cell-empty">-</span>
		{/if}
	{:else if col.type === 'bool'}
		{@const v = getCellValue(item, col) as number}
		{#if v > 0}
			<span class="cell-bool yes">✓</span>
		{:else}
			<span class="cell-bool no">✗</span>
		{/if}
	{:else if col.type === 'bar'}
		{@const v = getCellValue(item, col) as number}
		{@const max = col.max ?? 10}
		{#if v > 0}
			<span class="cell-bar">
				<span class="bar-track">
					<span class="bar-fill" style="width: {Math.min(v / max * 100, 100)}%"></span>
				</span>
				<span class="bar-val">{v}</span>
			</span>
		{:else}
			<span class="cell-empty">-</span>
		{/if}
	{:else if col.type === 'stars'}
		{@const v = getCellValue(item, col) as number}
		{#if v > 0}
			<span class="cell-stars">{'★'.repeat(Math.min(v, 5))}{'☆'.repeat(Math.max(5 - v, 0))}</span>
		{:else}
			<span class="cell-empty">-</span>
		{/if}
	{/if}
</span>

<style>
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
</style>

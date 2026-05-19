<script lang="ts">
	import type { Item, TripItemEnriched } from '$lib/types';
	import { itemName, itemBrand, itemModel } from '$lib/types';
	import { startDrag, endDrag } from '$lib/stores/dragState.svelte';

	let { item, alreadyAdded = false,
		onclick = null,
		onHover = null,
		itemId = 0, typeId = null, enrichedItems = []
	}: {
		item: Item;
		alreadyAdded?: boolean;
		onclick?: (() => void) | null;
		onHover?: ((itemId: number | null) => void) | null;
		itemId?: number;
		typeId?: number | null;
		enrichedItems?: TripItemEnriched[];
	} = $props();

	const displayName = $derived(itemName(item));
	const displayBrand = $derived(itemBrand(item));
	const displayModel = $derived(itemModel(item));
	const displayQty = $derived(Number(item.attrs?.default_qty ?? 1));

	const isDraggable = $derived(itemId > 0);
	let dragging = $state(false);

	function handleDragStart(e: DragEvent) {
		if (!isDraggable) { e.preventDefault(); return; }
		dragging = true;
		const data = { itemId, typeId };
		e.dataTransfer!.setData('application/json', JSON.stringify(data));
		e.dataTransfer!.effectAllowed = 'copy';
		startDrag(data, enrichedItems);
	}

	function handleDragEnd() {
		dragging = false;
		endDrag();
	}

	function handleClick() {
		onclick?.();
	}
</script>

<div
	class="item-card"
	class:already-added={alreadyAdded}
	class:dragging
	draggable={isDraggable ? 'true' : undefined}
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	onclick={onclick ? handleClick : undefined}
	onmouseenter={onHover ? () => onHover(itemId) : undefined}
	onmouseleave={onHover ? () => onHover(null) : undefined}
	role={onclick ? 'button' : undefined}
	tabindex={onclick ? 0 : undefined}
	onkeydown={onclick ? (e: KeyboardEvent) => { if (e.key === 'Enter' || e.key === ' ') handleClick(); } : undefined}
>
	<div class="card-name">{displayName}</div>
	{#if displayBrand || displayModel}
		<div class="card-detail">{displayBrand} {displayModel}</div>
	{/if}
	{#if displayQty > 1 || displayQty === 0}
		<div class="card-qty" class:zero={displayQty === 0}>x{displayQty}</div>
	{/if}
	{#if alreadyAdded}
		<div class="added-tag">已添加</div>
	{/if}
</div>

<style>
	.item-card {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 10px 8px;
		text-align: center;
		cursor: default;
		transition: all 0.2s;
		position: relative;
		min-height: 90px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		user-select: none;
	}
	.item-card[draggable='true'] {
		cursor: grab;
	}
	.item-card:hover {
		border-color: var(--primary);
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}
	.item-card[draggable='true']:active {
		cursor: grabbing;
	}
	.item-card.dragging {
		opacity: 0.4;
		cursor: grabbing;
	}
	.item-card.already-added {
		opacity: 0.5;
	}
	.card-name {
		font-size: 13px;
		color: var(--text);
		font-weight: 500;
		line-height: 1.2;
	}
	.card-detail {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.card-qty {
		position: absolute;
		top: 4px;
		right: 6px;
		background: var(--primary);
		color: white;
		font-size: 10px;
		padding: 0 5px;
		border-radius: 8px;
		font-weight: 600;
	}
	.card-qty.zero {
		background: var(--text-secondary);
		opacity: 0.5;
	}
	.added-tag {
		font-size: 10px;
		color: var(--text-secondary);
		background: var(--bg);
		padding: 0 6px;
		border-radius: 4px;
		margin-top: 2px;
	}
</style>

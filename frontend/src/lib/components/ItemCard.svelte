<script lang="ts">
	import type { TripItemEnriched } from '$lib/types';
	import { startDrag, endDrag } from '$lib/stores/dragState.svelte';

	let { name, brand, model, categoryIcon, qty, alreadyAdded = false,
		itemId = 0, tagId = null, enrichedItems = [], onclick
	}: {
		name: string;
		brand: string;
		model: string;
		categoryIcon: string;
		qty: number;
		alreadyAdded?: boolean;
		itemId?: number;
		tagId?: number | null;
		enrichedItems?: TripItemEnriched[];
		onclick?: () => void;
	} = $props();

	const isDraggable = $derived(itemId > 0 && enrichedItems.length > 0);
	let dragging = $state(false);

	function handleDragStart(e: DragEvent) {
		if (!isDraggable) { e.preventDefault(); return; }
		dragging = true;
		const data = { itemId, tagId };
		e.dataTransfer!.setData('application/json', JSON.stringify(data));
		e.dataTransfer!.effectAllowed = 'copy';
		startDrag(data, enrichedItems);
	}

	function handleDragEnd() {
		dragging = false;
		endDrag();
	}
</script>

<div
	class="item-card"
	class:already-added={alreadyAdded}
	class:dragging
	class:clickable={!!onclick}
	draggable={isDraggable ? 'true' : undefined}
	ondragstart={handleDragStart}
	ondragend={handleDragEnd}
	onclick={onclick}
	role={onclick ? 'button' : undefined}
	tabindex={onclick ? 0 : undefined}
	onkeydown={onclick ? (e) => e.key === 'Enter' && onclick?.() : undefined}
>
	<div class="card-icon">{categoryIcon}</div>
	<div class="card-name">{name}</div>
	{#if brand || model}
		<div class="card-detail">{brand} {model}</div>
	{/if}
	{#if qty > 1}
		<div class="card-qty">x{qty}</div>
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
	.item-card.clickable {
		cursor: pointer;
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
	.card-icon {
		font-size: 22px;
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
	.added-tag {
		font-size: 10px;
		color: var(--text-secondary);
		background: var(--bg);
		padding: 0 6px;
		border-radius: 4px;
		margin-top: 2px;
	}
</style>

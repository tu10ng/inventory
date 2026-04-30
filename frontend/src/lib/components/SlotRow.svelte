<script lang="ts">
	import type { TripItemEnriched, Person, ItemStatus } from '$lib/types';
	import StatusBadge from './StatusBadge.svelte';
	import PersonBadge from './PersonBadge.svelte';
	import InlineEdit from './InlineEdit.svelte';

	let { tripItem, people, selected = false, selectable = false,
		isDragging = false, isValidDropTarget = false,
		onToggleCheck, onUpdateStatus, onUpdateQty, onUpdateNotes, onUpdatePerson, onToggleSelect,
		onAssignItem, onClearItem
	}: {
		tripItem: TripItemEnriched;
		people: Person[];
		selected?: boolean;
		selectable?: boolean;
		isDragging?: boolean;
		isValidDropTarget?: boolean;
		onToggleCheck: () => void;
		onUpdateStatus: (s: ItemStatus) => void;
		onUpdateQty: (q: number) => void;
		onUpdateNotes: (n: string) => void;
		onUpdatePerson: (id: number | null) => void;
		onToggleSelect?: () => void;
		onAssignItem: (newItemId: number) => void;
		onClearItem: () => void;
	} = $props();

	const currentItem = $derived(
		tripItem.candidates.find(c => c.id === tripItem.item_id) ?? null
	);

	const categoryMismatch = $derived.by(() => {
		if (!currentItem || !tripItem.slot) return false;
		return currentItem.category_id !== tripItem.slot.category_id;
	});

	// Whether all candidates share the same tag (single-type slot)
	const isSingleType = $derived.by(() => {
		if (tripItem.candidates.length <= 1) return true;
		const tags = new Set(tripItem.candidates.map(c => c.tag_id));
		return tags.size === 1;
	});

	let dragOver = $state(false);

	function handleDragOver(e: DragEvent) {
		if (!isValidDropTarget) return;
		e.preventDefault();
		e.dataTransfer!.dropEffect = 'copy';
		dragOver = true;
	}

	function handleDragLeave() {
		dragOver = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragOver = false;
		if (!isValidDropTarget) return;
		try {
			const data = JSON.parse(e.dataTransfer!.getData('application/json'));
			if (data.itemId) onAssignItem(data.itemId);
		} catch { /* ignore bad data */ }
	}
</script>

<div
	class="slot-item"
	class:checked={tripItem.checked}
	class:selected
	class:drag-valid={isDragging && isValidDropTarget && !dragOver}
	class:drag-over={dragOver}
	class:drag-invalid={isDragging && !isValidDropTarget}
	role="listitem"
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
>
	<div class="item-left">
		{#if selectable}
			<input
				type="checkbox"
				class="select-check"
				checked={selected}
				onchange={() => onToggleSelect?.()}
				onclick={(e) => e.stopPropagation()}
			/>
		{/if}
		<label class="check-label">
			<input
				type="checkbox"
				checked={tripItem.checked}
				onchange={onToggleCheck}
			/>
			<span class="item-name" class:line-through={tripItem.checked}>
				{#if tripItem.is_essential}<span class="essential-star" title="必备">★</span>{/if}
				{tripItem.slot?.slot_name ?? tripItem.custom_name ?? '未知'}
			</span>
		</label>
	</div>

	<div class="item-meta">
		<!-- Desktop: pill/badge or drop target -->
		<div class="slot-assignment desktop-only">
			{#if currentItem}
				<span class="item-pill">
					{#if isSingleType}
						<span class="pill-brand">{currentItem.brand || currentItem.name}</span>
					{:else}
						<span class="pill-name">{currentItem.name}</span>
						{#if currentItem.brand}
							<span class="pill-brand">{currentItem.brand}</span>
						{/if}
					{/if}
				</span>
				<button class="clear-btn" onclick={onClearItem} title="清空物品选择">×</button>
			{:else if tripItem.candidates.length > 0}
				<span class="drop-target" class:drop-highlight={dragOver}>
					拖入物品...
				</span>
			{:else}
				<span class="no-candidates">暂无匹配物品</span>
			{/if}
			{#if categoryMismatch}
				<span class="mismatch-warn" title="物品分类与槽位分类不一致">⚠️</span>
			{/if}
		</div>

		<!-- Mobile: select dropdown fallback -->
		<div class="mobile-only">
			{#if tripItem.candidates.length > 0}
				<select
					class="candidate-select"
					value={tripItem.item_id ?? ''}
					onchange={(e) => {
						const val = e.currentTarget.value;
						if (val) onAssignItem(Number(val));
						else onClearItem();
					}}
				>
					<option value="">选择物品...</option>
					{#each tripItem.candidates as c}
						<option value={c.id}>
							{c.name}{c.brand ? ` ${c.brand}` : ''}{c.model ? ` ${c.model}` : ''}
						</option>
					{/each}
				</select>
			{:else}
				<span class="no-candidates">暂无匹配</span>
			{/if}
		</div>

		<InlineEdit
			value={tripItem.qty}
			type="number"
			min={1}
			oncommit={(v) => onUpdateQty(Number(v))}
		/>
		<StatusBadge
			status={tripItem.item_status}
			onchange={onUpdateStatus}
		/>
		<PersonBadge
			personId={tripItem.person_id}
			{people}
			onchange={onUpdatePerson}
		/>
	</div>
</div>

<style>
	.slot-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 16px;
		border-top: 1px solid var(--border);
		transition: background 0.15s, border-color 0.15s;
	}
	.slot-item:hover {
		background: var(--bg);
	}
	.slot-item.checked {
		opacity: 0.6;
	}
	.slot-item.selected {
		background: #e8f0fe;
	}
	.slot-item.drag-valid {
		background: #f0faf0;
		border-color: #4caf50;
	}
	.slot-item.drag-over {
		background: #e0f5e0;
		border-color: #4caf50;
		box-shadow: inset 0 0 0 2px #4caf50;
	}
	.slot-item.drag-invalid {
		opacity: 0.4;
	}
	.item-left {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}
	.check-label {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}
	.check-label input[type='checkbox'] {
		width: 18px;
		height: 18px;
		cursor: pointer;
	}
	.select-check {
		width: 16px;
		height: 16px;
		cursor: pointer;
		accent-color: var(--primary);
	}
	.item-name {
		font-size: 15px;
		white-space: nowrap;
	}
	.line-through {
		text-decoration: line-through;
	}
	.essential-star {
		color: var(--warning);
		margin-right: 2px;
	}
	.item-meta {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
	}
	.slot-assignment {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.item-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: #eef2ff;
		border: 1px solid #c7d2fe;
		border-radius: 12px;
		font-size: 13px;
		white-space: nowrap;
	}
	.pill-name {
		color: var(--text);
	}
	.pill-brand {
		background: #818cf8;
		color: white;
		padding: 0 6px;
		border-radius: 8px;
		font-size: 11px;
		font-weight: 500;
	}
	.clear-btn {
		background: #f1f5f9;
		border: 1px solid #cbd5e1;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: 14px;
		padding: 0;
		line-height: 1;
		border-radius: 50%;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	.clear-btn:hover {
		background: var(--danger);
		border-color: var(--danger);
		color: white;
	}
	.drop-target {
		display: inline-block;
		padding: 2px 10px;
		border: 2px dashed var(--border);
		border-radius: 12px;
		color: var(--text-secondary);
		font-size: 12px;
		white-space: nowrap;
		transition: all 0.15s;
	}
	.drop-target.drop-highlight {
		border-color: #4caf50;
		background: #e8f5e9;
		color: #2e7d32;
	}
	.no-candidates {
		color: var(--text-secondary);
		font-style: italic;
		font-size: 12px;
	}
	.mismatch-warn {
		cursor: help;
	}
	.candidate-select {
		padding: 2px 6px;
		font-size: 13px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: white;
		max-width: 180px;
	}

	/* Desktop/mobile toggle */
	.desktop-only {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.mobile-only {
		display: none;
	}
	@media (max-width: 768px) {
		.desktop-only {
			display: none;
		}
		.mobile-only {
			display: flex;
			align-items: center;
			gap: 6px;
		}
	}
</style>

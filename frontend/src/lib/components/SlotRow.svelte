<script lang="ts">
	import type { TripItemEnriched, Item, Person, ItemStatus } from '$lib/types';
	import StatusBadge from './StatusBadge.svelte';
	import PersonBadge from './PersonBadge.svelte';
	import InlineEdit from './InlineEdit.svelte';

	let { tripItem, people, selected = false, selectable = false,
		onToggleCheck, onUpdateStatus, onUpdateQty, onUpdateNotes, onUpdatePerson, onRemove, onToggleSelect, onChangeItem
	}: {
		tripItem: TripItemEnriched;
		people: Person[];
		selected?: boolean;
		selectable?: boolean;
		onToggleCheck: () => void;
		onUpdateStatus: (s: ItemStatus) => void;
		onUpdateQty: (q: number) => void;
		onUpdateNotes: (n: string) => void;
		onUpdatePerson: (id: number | null) => void;
		onRemove: () => void;
		onToggleSelect?: () => void;
		onChangeItem: (newItemId: number) => void;
	} = $props();

	const currentItem = $derived(
		tripItem.candidates.find(c => c.id === tripItem.item_id) ?? null
	);

	const categoryMismatch = $derived.by(() => {
		if (!currentItem || !tripItem.slot) return false;
		return currentItem.category_id !== tripItem.slot.category_id;
	});
</script>

<div class="slot-item" class:checked={tripItem.checked} class:selected>
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
		{#if tripItem.candidates.length > 0}
			<select
				class="candidate-select"
				value={tripItem.item_id ?? ''}
				onchange={(e) => {
					const val = e.currentTarget.value;
					if (val) onChangeItem(Number(val));
				}}
			>
				{#if !tripItem.item_id}
					<option value="">选择物品...</option>
				{/if}
				{#each tripItem.candidates as c}
					<option value={c.id}>
						{c.name}{c.brand ? ` ${c.brand}` : ''}{c.model ? ` ${c.model}` : ''}
					</option>
				{/each}
			</select>
			{#if categoryMismatch}
				<span class="mismatch-warn" title="物品分类与槽位分类不一致">⚠️</span>
			{/if}
		{:else}
			<span class="no-candidates">暂无匹配物品</span>
		{/if}

		{#if currentItem?.brand || currentItem?.model}
			<span class="item-detail">{currentItem?.brand} {currentItem?.model}</span>
		{/if}

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
		<button class="small danger" onclick={onRemove}>×</button>
	</div>
</div>

<style>
	.slot-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 16px;
		border-top: 1px solid var(--border);
		transition: background 0.15s;
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
	.candidate-select {
		padding: 2px 6px;
		font-size: 13px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: white;
		max-width: 180px;
	}
	.no-candidates {
		color: var(--text-secondary);
		font-style: italic;
		font-size: 12px;
	}
	.mismatch-warn {
		cursor: help;
	}
	.item-detail {
		color: var(--text-secondary);
		white-space: nowrap;
	}
</style>

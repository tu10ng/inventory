<script lang="ts">
	import type { TripItem, Item, Person, ItemStatus } from '$lib/types';
	import StatusBadge from './StatusBadge.svelte';
	import PersonBadge from './PersonBadge.svelte';
	import InlineEdit from './InlineEdit.svelte';

	let { tripItem, itemInfo, people, selected = false, selectable = false,
		canSaveAsSlot = false,
		onToggleCheck, onUpdateStatus, onUpdateQty, onUpdateNotes, onUpdatePerson, onRemove, onToggleSelect,
		onSaveAsSlot
	}: {
		tripItem: TripItem;
		itemInfo: Item | null;
		people: Person[];
		selected?: boolean;
		selectable?: boolean;
		canSaveAsSlot?: boolean;
		onToggleCheck: () => void;
		onUpdateStatus: (s: ItemStatus) => void;
		onUpdateQty: (q: number) => void;
		onUpdateNotes: (n: string) => void;
		onUpdatePerson: (id: number | null) => void;
		onRemove: () => void;
		onToggleSelect?: () => void;
		onSaveAsSlot?: () => void;
	} = $props();
</script>

<div class="checklist-item" class:checked={tripItem.checked} class:selected>
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
				{tripItem.custom_name || itemInfo?.name || '未知物品'}
			</span>
		</label>
	</div>

	<div class="item-meta">
		{#if itemInfo?.brand || itemInfo?.model}
			<span class="item-detail">{itemInfo?.brand} {itemInfo?.model}</span>
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
		<InlineEdit
			value={tripItem.notes}
			placeholder="备注..."
			oncommit={(v) => onUpdateNotes(String(v))}
		/>
		{#if canSaveAsSlot && itemInfo}
			<button class="small save-slot-btn" onclick={onSaveAsSlot} title="保存到模板">↑</button>
		{/if}
		<button class="small danger" onclick={onRemove}>×</button>
	</div>
</div>

<style>
	.checklist-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 16px;
		border-top: 1px solid var(--border);
		transition: background 0.15s;
	}
	.checklist-item:hover {
		background: var(--bg);
	}
	.checklist-item.checked {
		opacity: 0.6;
	}
	.checklist-item.selected {
		background: #e8f0fe;
	}
	.item-left {
		display: flex;
		align-items: center;
		gap: 8px;
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
	.item-detail {
		color: var(--text-secondary);
	}
	.save-slot-btn {
		color: var(--primary);
		border-color: var(--primary);
		font-weight: 600;
	}
	.save-slot-btn:hover {
		background: var(--primary);
		color: white;
	}
</style>

<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/client';
	import type { Trip, TripItem, TripItemEnriched, Item, Category, Tip, Person, ResyncPreview, ResyncPreviewItem } from '$lib/types';
	import { TRIP_STATUS_LABELS } from '$lib/utils/status';
	import SplitPane from '$lib/components/SplitPane.svelte';
	import ChecklistPanel from '$lib/components/ChecklistPanel.svelte';
	import InventoryPanel from '$lib/components/InventoryPanel.svelte';

	let trip = $state<Trip | null>(null);
	let enrichedItems = $state<TripItemEnriched[]>([]);
	let allItems = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tips = $state<Tip[]>([]);
	let people = $state<Person[]>([]);

	const tripId = $derived(Number(page.params.id));

	const tripItemIds = $derived(new Set(enrichedItems.filter((ti) => ti.item_id).map((ti) => ti.item_id!)));

	async function load() {
		const id = tripId;
		const [t, items, cats, ppl] = await Promise.all([
			api.get<Trip>(`/trips/${id}`),
			api.get<Item[]>('/items'),
			api.get<Category[]>('/categories'),
			api.get<Person[]>('/people')
		]);
		trip = t;
		allItems = items;
		categories = cats;
		people = ppl;

		// Load enriched items
		enrichedItems = await api.get<TripItemEnriched[]>(`/trips/${id}/items/enriched`);

		if (t.activity_id) {
			try {
				tips = await api.get<Tip[]>(`/activities/${t.activity_id}/tips`);
			} catch {
				// tips not critical
			}
		}
	}

	async function reloadItems() {
		enrichedItems = await api.get<TripItemEnriched[]>(`/trips/${tripId}/items/enriched`);
	}

	async function populate() {
		await api.post<TripItem[]>(`/trips/${tripId}/populate`);
		await reloadItems();
	}

	function previewItemLabel(item: ResyncPreviewItem): string {
		const name = item.item_name || item.custom_name || item.slot_name || '未知物品';
		return `  - ${name}（${item.reason}）`;
	}

	async function resync() {
		const preview = await api.post<ResyncPreview>(`/trips/${tripId}/resync-preview`);
		if (preview.items_to_remove.length === 0 && preview.items_to_add.length === 0) {
			alert('模板没有变化，无需同步。');
			return;
		}

		const lines: string[] = ['同步模板将执行以下操作：\n'];
		if (preview.items_to_remove.length > 0) {
			lines.push(`移除 ${preview.items_to_remove.length} 项：`);
			for (const item of preview.items_to_remove) {
				lines.push(previewItemLabel(item));
			}
		}
		if (preview.items_to_add.length > 0) {
			if (preview.items_to_remove.length > 0) lines.push('');
			lines.push(`新增 ${preview.items_to_add.length} 项：`);
			for (const item of preview.items_to_add) {
				lines.push(previewItemLabel(item));
			}
		}
		lines.push('\n确定执行吗？');

		if (!window.confirm(lines.join('\n'))) return;

		await api.post<TripItem[]>(`/trips/${tripId}/resync`);
		await reloadItems();
	}

	async function updateTripStatus(status: string) {
		if (!trip) return;
		trip = await api.put<Trip>(`/trips/${tripId}`, { ...trip, status });
	}

	async function cloneTrip() {
		const newTrip = await api.post<Trip>(`/trips/${tripId}/clone`);
		goto(`/trips/${newTrip.id}`);
	}

	$effect(() => {
		load();
	});
</script>

{#if trip}
	<div class="trip-header">
		<div>
			<h1>{trip.name}</h1>
			{#if trip.start_date}
				<div class="trip-dates">
					{trip.start_date}{#if trip.end_date} ~ {trip.end_date}{/if}
				</div>
			{/if}
		</div>
		<div class="trip-controls">
			<select
				value={trip.status}
				onchange={(e) => updateTripStatus(e.currentTarget.value)}
			>
				<option value="planning">计划中</option>
				<option value="packing">打包中</option>
				<option value="done">已完成</option>
			</select>
			<span class="badge {trip.status}">{TRIP_STATUS_LABELS[trip.status]}</span>
			<button class="small no-print" onclick={cloneTrip} title="克隆行程">📋 克隆</button>
		</div>
	</div>

	<SplitPane>
		{#snippet left()}
			<ChecklistPanel
				trip={trip!}
				bind:enrichedItems
				{allItems}
				{categories}
				{tips}
				{people}
				onPopulate={populate}
				onResync={resync}
				onReload={reloadItems}
			/>
		{/snippet}
		{#snippet right()}
			<InventoryPanel
				items={allItems}
				{categories}
				{tripItemIds}
				{enrichedItems}
			/>
		{/snippet}
	</SplitPane>
{:else}
	<p>加载中...</p>
{/if}

<style>
	.trip-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 16px;
	}
	.trip-dates {
		color: var(--text-secondary);
		margin-top: 4px;
	}
	.trip-controls {
		display: flex;
		gap: 8px;
		align-items: center;
	}
</style>

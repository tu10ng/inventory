<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/client';
	import type { Trip, TripItem, Item, Category, Tip, Person } from '$lib/types';
	import { TRIP_STATUS_LABELS } from '$lib/utils/status';
	import SplitPane from '$lib/components/SplitPane.svelte';
	import ChecklistPanel from '$lib/components/ChecklistPanel.svelte';
	import InventoryPanel from '$lib/components/InventoryPanel.svelte';

	let trip = $state<Trip | null>(null);
	let tripItems = $state<TripItem[]>([]);
	let allItems = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tips = $state<Tip[]>([]);
	let people = $state<Person[]>([]);

	const tripId = $derived(Number(page.params.id));

	const tripItemIds = $derived(new Set(tripItems.filter((ti) => ti.item_id).map((ti) => ti.item_id!)));

	async function load() {
		const id = tripId;
		const [t, ti, items, cats, ppl] = await Promise.all([
			api.get<Trip>(`/trips/${id}`),
			api.get<TripItem[]>(`/trips/${id}/items`),
			api.get<Item[]>('/items'),
			api.get<Category[]>('/categories'),
			api.get<Person[]>('/people')
		]);
		trip = t;
		tripItems = ti;
		allItems = items;
		categories = cats;
		people = ppl;

		if (t.activity_id) {
			tips = await api.get<Tip[]>(`/activities/${t.activity_id}/tips`);
		}
	}

	async function populate() {
		tripItems = await api.post<TripItem[]>(`/trips/${tripId}/populate`);
	}

	async function resync() {
		tripItems = await api.post<TripItem[]>(`/trips/${tripId}/resync`);
	}

	async function updateTripStatus(status: string) {
		if (!trip) return;
		trip = await api.put<Trip>(`/trips/${tripId}`, { ...trip, status });
	}

	async function cloneTrip() {
		const newTrip = await api.post<Trip>(`/trips/${tripId}/clone`);
		goto(`/trips/${newTrip.id}`);
	}

	async function addItemFromInventory(itemId: number) {
		const item = allItems.find((i) => i.id === itemId);
		await api.post(`/trips/${tripId}/items`, {
			item_id: itemId,
			qty: item?.default_qty ?? 1
		});
		tripItems = await api.get<TripItem[]>(`/trips/${tripId}/items`);
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
				bind:tripItems
				{allItems}
				{categories}
				{tips}
				{people}
				onPopulate={populate}
				onResync={resync}
			/>
		{/snippet}
		{#snippet right()}
			<InventoryPanel
				items={allItems}
				{categories}
				{tripItemIds}
				onAddItem={addItemFromInventory}
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

<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/client';
	import type { Trip, TripItem, TripItemEnriched, Item, Category, Tip, Person, ResyncPreview, ResyncPreviewItem, StatusDefinition, ItemRelationEnriched } from '$lib/types';
	import { getItemStatuses, getTripStatuses, getTripStatusLabel } from '$lib/utils/status';
	import SplitPane from '$lib/components/SplitPane.svelte';
	import ChecklistPanel from '$lib/components/ChecklistPanel.svelte';
	import InventoryPanel from '$lib/components/InventoryPanel.svelte';

	let trip = $state<Trip | null>(null);
	let enrichedItems = $state<TripItemEnriched[]>([]);
	let allItems = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tips = $state<Tip[]>([]);
	let people = $state<Person[]>([]);
	let itemStatusDefs = $state<StatusDefinition[]>([]);
	let tripStatusDefs = $state<StatusDefinition[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	const tripId = $derived(Number(page.params.id));

	const tripItemIds = $derived(new Set(enrichedItems.filter((ti) => ti.item_id).map((ti) => ti.item_id!)));

	// Relation recommendations
	let hoveredItemId = $state<number | null>(null);
	let hoveredRelations = $state<ItemRelationEnriched[]>([]);
	let relationLoading = $state(false);

	async function handleHoverItem(itemId: number | null) {
		if (hoveredItemId === itemId) return;
		hoveredItemId = itemId;
		hoveredRelations = [];
		if (itemId === null) return;

		relationLoading = true;
		try {
			hoveredRelations = await api.get<ItemRelationEnriched[]>(`/items/${itemId}/relations`);
		} catch {
			// not critical
		} finally {
			relationLoading = false;
		}
	}

	// Related items that are NOT already in the trip
	const recommendedItems = $derived(
		hoveredRelations
			.filter(r => !tripItemIds.has(r.target_item_id))
			.map(r => {
				const item = allItems.find(i => i.id === r.target_item_id);
				return item ? { relation: r, item } : null;
			})
			.filter((x): x is NonNullable<typeof x> => x !== null)
	);

	async function load() {
		try {
			loading = true;
			error = null;
			const id = tripId;
			const [t, items, cats, ppl, iDefs, tDefs] = await Promise.all([
				api.get<Trip>(`/trips/${id}`),
				api.get<Item[]>('/items'),
				api.get<Category[]>('/categories'),
				api.get<Person[]>('/people'),
				getItemStatuses(),
				getTripStatuses()
			]);
			trip = t;
			allItems = items;
			categories = cats;
			people = ppl;
			itemStatusDefs = iDefs;
			tripStatusDefs = tDefs;

			// Load enriched items
			enrichedItems = await api.get<TripItemEnriched[]>(`/trips/${id}/items/enriched`);

			if (t.activity_id) {
				try {
					tips = await api.get<Tip[]>(`/activities/${t.activity_id}/tips`);
				} catch {
					// tips not critical
				}
			}
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loading = false;
		}
	}

	async function reloadItems() {
		enrichedItems = await api.get<TripItemEnriched[]>(`/trips/${tripId}/items/enriched`);
	}

	async function populate() {
		try {
			await api.post<TripItem[]>(`/trips/${tripId}/populate`);
			await reloadItems();
		} catch (e) {
			alert((e as Error).message);
		}
	}

	function previewItemLabel(item: ResyncPreviewItem): string {
		const name = item.item_name || item.custom_name || item.slot_name || '未知物品';
		return `  - ${name}（${item.reason}）`;
	}

	async function resync() {
		try {
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
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function updateTripStatus(status: string) {
		if (!trip) return;
		try {
			trip = await api.put<Trip>(`/trips/${tripId}`, { ...trip, status });
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function cloneTrip() {
		try {
			const newTrip = await api.post<Trip>(`/trips/${tripId}/clone`);
			goto(`/trips/${newTrip.id}`);
		} catch (e) {
			alert((e as Error).message);
		}
	}

	$effect(() => {
		load();
	});
</script>

{#if loading}
	<div class="loading-state">加载中...</div>
{:else if error}
	<div class="error-state">
		<p>{error}</p>
		<button onclick={load}>重试</button>
	</div>
{:else if trip}
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
				{#each tripStatusDefs as sd}
					<option value={sd.value}>{sd.label}</option>
				{/each}
			</select>
			<span class="badge {trip.status}">{getTripStatusLabel(trip.status)}</span>
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
				statusDefs={itemStatusDefs}
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
				onHoverItem={handleHoverItem}
			/>
			{#if recommendedItems.length > 0}
				<div class="recommendation-bar card">
					<div class="rec-header">关联推荐</div>
					<div class="rec-chips">
						{#each recommendedItems as { relation, item } (item.id)}
							<span class="rec-chip" title="{relation.relation_label}: {relation.relation_icon}">
								{relation.relation_icon} {String(item.attrs?.name ?? '?')}
							</span>
						{/each}
					</div>
				</div>
			{/if}
		{/snippet}
	</SplitPane>
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
	.loading-state {
		text-align: center;
		padding: 40px;
		color: var(--text-secondary);
	}
	.error-state {
		text-align: center;
		padding: 40px;
		color: var(--danger);
	}
	.error-state button {
		margin-top: 12px;
	}
	.recommendation-bar {
		margin-top: 12px;
		padding: 10px 12px;
	}
	.rec-header {
		font-size: 12px;
		font-weight: 600;
		color: var(--text-secondary);
		margin-bottom: 6px;
		text-transform: uppercase;
	}
	.rec-chips {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	.rec-chip {
		font-size: 12px;
		background: var(--primary);
		color: white;
		padding: 3px 10px;
		border-radius: 12px;
		cursor: default;
	}
</style>

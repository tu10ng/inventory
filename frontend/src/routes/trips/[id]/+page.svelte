<script lang="ts">
	import { page } from '$app/state';
	import { api } from '$lib/api/client';
	import type { Trip, TripItem, Item, Category, Tip, Person } from '$lib/types';
	import { STATUS_LABELS, STATUS_OPTIONS, TRIP_STATUS_LABELS } from '$lib/utils/status';
	import type { ItemStatus } from '$lib/types';

	let trip = $state<Trip | null>(null);
	let tripItems = $state<TripItem[]>([]);
	let allItems = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tips = $state<Tip[]>([]);
	let people = $state<Person[]>([]);
	let collapsed = $state<Record<number, boolean>>({});

	// Add item form
	let showAddForm = $state(false);
	let addItemId = $state<number | null>(null);
	let addCustomName = $state('');
	let addQty = $state(1);

	const tripId = $derived(Number(page.params.id));

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

	async function toggleCheck(ti: TripItem) {
		const updated = await api.patch<TripItem>(`/trip-items/${ti.id}/check`, {
			checked: !ti.checked
		});
		tripItems = tripItems.map((t) => (t.id === updated.id ? updated : t));
	}

	async function updateStatus(ti: TripItem, status: ItemStatus) {
		const updated = await api.put<TripItem>(`/trip-items/${ti.id}`, {
			...ti,
			item_status: status
		});
		tripItems = tripItems.map((t) => (t.id === updated.id ? updated : t));
	}

	async function updateTripStatus(status: string) {
		if (!trip) return;
		trip = await api.put<Trip>(`/trips/${tripId}`, { ...trip, status });
	}

	async function removeTripItem(id: number) {
		await api.del(`/trip-items/${id}`);
		tripItems = tripItems.filter((t) => t.id !== id);
	}

	async function addTripItem() {
		const body: Record<string, unknown> = { qty: addQty };
		if (addItemId) body.item_id = addItemId;
		if (addCustomName) body.custom_name = addCustomName;
		await api.post(`/trips/${tripId}/items`, body);
		addItemId = null;
		addCustomName = '';
		addQty = 1;
		showAddForm = false;
		tripItems = await api.get<TripItem[]>(`/trips/${tripId}/items`);
	}

	function getItemInfo(itemId: number | null) {
		if (!itemId) return null;
		return allItems.find((i) => i.id === itemId) ?? null;
	}

	function getCategoryForItem(itemId: number | null): Category | null {
		const item = getItemInfo(itemId);
		if (!item) return null;
		return categories.find((c) => c.id === item.category_id) ?? null;
	}

	function toggleCollapse(catId: number) {
		collapsed[catId] = !collapsed[catId];
	}

	// Group trip items by category
	const groupedItems = $derived.by(() => {
		const groups: { category: Category; items: TripItem[] }[] = [];
		const uncategorized: TripItem[] = [];

		const catMap = new Map<number, TripItem[]>();
		for (const ti of tripItems) {
			const cat = getCategoryForItem(ti.item_id);
			if (cat) {
				if (!catMap.has(cat.id)) catMap.set(cat.id, []);
				catMap.get(cat.id)!.push(ti);
			} else {
				uncategorized.push(ti);
			}
		}

		for (const cat of categories) {
			const items = catMap.get(cat.id);
			if (items && items.length > 0) {
				groups.push({ category: cat, items });
			}
		}

		if (uncategorized.length > 0) {
			groups.push({
				category: { id: -1, name: '其他', icon: '📦', sort_order: 999 },
				items: uncategorized
			});
		}

		return groups;
	});

	const totalChecked = $derived(tripItems.filter((t) => t.checked).length);
	const totalItems = $derived(tripItems.length);
	const progress = $derived(totalItems > 0 ? Math.round((totalChecked / totalItems) * 100) : 0);

	$effect(() => {
		load();
	});
</script>

{#if trip}
	<div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 8px;">
		<div>
			<h1>{trip.name}</h1>
			{#if trip.start_date}
				<div style="color: var(--text-secondary); margin-top: 4px;">
					{trip.start_date}{#if trip.end_date} ~ {trip.end_date}{/if}
				</div>
			{/if}
		</div>
		<div style="display: flex; gap: 8px; align-items: center;">
			<select
				value={trip.status}
				onchange={(e) => updateTripStatus(e.currentTarget.value)}
			>
				<option value="planning">计划中</option>
				<option value="packing">打包中</option>
				<option value="done">已完成</option>
			</select>
			<span class="badge {trip.status}">{TRIP_STATUS_LABELS[trip.status]}</span>
		</div>
	</div>

	<!-- Progress bar -->
	{#if totalItems > 0}
		<div style="margin-bottom: 16px;">
			<div style="display: flex; justify-content: space-between; font-size: 14px; color: var(--text-secondary); margin-bottom: 4px;">
				<span>进度</span>
				<span>{totalChecked}/{totalItems} ({progress}%)</span>
			</div>
			<div style="height: 8px; background: var(--border); border-radius: 4px; overflow: hidden;">
				<div style="height: 100%; width: {progress}%; background: var(--success); transition: width 0.3s;"></div>
			</div>
		</div>
	{/if}

	<!-- Tips -->
	{#if tips.length > 0}
		<div class="card" style="background: #fff3cd; border-color: #ffc107; margin-bottom: 16px;">
			{#each tips as tip}
				<div style="padding: 2px 0;">⚠️ {tip.content}</div>
			{/each}
		</div>
	{/if}

	<!-- Actions -->
	<div style="display: flex; gap: 8px; margin-bottom: 16px;">
		{#if trip.activity_id && tripItems.length === 0}
			<button class="primary" onclick={populate}>从模板填充清单</button>
		{/if}
		<button onclick={() => (showAddForm = !showAddForm)}>
			{showAddForm ? '取消' : '+ 添加物品'}
		</button>
	</div>

	{#if showAddForm}
		<div class="card" style="margin-bottom: 16px;">
			<div style="display: flex; gap: 10px; align-items: end;">
				<div style="flex: 1;">
					<div style="font-size: 13px; color: var(--text-secondary); margin-bottom: 4px;">从物品库选择</div>
					<select bind:value={addItemId} style="width: 100%;">
						<option value={null}>选择物品...</option>
						{#each allItems as it}
							<option value={it.id}>{it.name} {it.brand} {it.model}</option>
						{/each}
					</select>
				</div>
				<div>
					<div style="font-size: 13px; color: var(--text-secondary); margin-bottom: 4px;">或自定义名称</div>
					<input bind:value={addCustomName} placeholder="自定义物品" />
				</div>
				<div>
					<div style="font-size: 13px; color: var(--text-secondary); margin-bottom: 4px;">数量</div>
					<input type="number" bind:value={addQty} min="1" style="width: 60px;" />
				</div>
				<button class="primary" onclick={addTripItem} disabled={!addItemId && !addCustomName}>添加</button>
			</div>
		</div>
	{/if}

	<!-- Checklist grouped by category -->
	{#each groupedItems as group}
		{@const checked = group.items.filter((t) => t.checked).length}
		<div class="card" style="padding: 0; overflow: hidden;">
			<div
				style="display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; cursor: pointer; background: {collapsed[group.category.id] ? 'var(--bg)' : 'transparent'};"
				onclick={() => toggleCollapse(group.category.id)}
				role="button"
				tabindex="0"
				onkeydown={(e) => e.key === 'Enter' && toggleCollapse(group.category.id)}
			>
				<strong>{group.category.icon} {group.category.name} ({checked}/{group.items.length})</strong>
				<span>{collapsed[group.category.id] ? '▶' : '▼'}</span>
			</div>

			{#if !collapsed[group.category.id]}
				{#each group.items as ti}
					{@const itemInfo = getItemInfo(ti.item_id)}
					<div class="checklist-item" class:checked={ti.checked}>
						<label class="check-label">
							<input
								type="checkbox"
								checked={ti.checked}
								onchange={() => toggleCheck(ti)}
							/>
							<span class="item-name" class:line-through={ti.checked}>
								{ti.custom_name || itemInfo?.name || '未知物品'}
							</span>
						</label>

						<div class="item-meta">
							{#if itemInfo?.brand || itemInfo?.model}
								<span class="item-detail">{itemInfo?.brand} {itemInfo?.model}</span>
							{/if}
							{#if ti.qty > 1}
								<span class="item-detail">x{ti.qty}</span>
							{/if}
							{#if ti.item_status}
								<select
									class="status-select badge {ti.item_status}"
									value={ti.item_status}
									onchange={(e) => updateStatus(ti, e.currentTarget.value as ItemStatus)}
									onclick={(e) => e.stopPropagation()}
								>
									{#each STATUS_OPTIONS as opt}
										<option value={opt.value}>{opt.label}</option>
									{/each}
								</select>
							{:else}
								<select
									class="status-select"
									value=""
									onchange={(e) => updateStatus(ti, e.currentTarget.value as ItemStatus)}
									onclick={(e) => e.stopPropagation()}
								>
									{#each STATUS_OPTIONS as opt}
										<option value={opt.value}>{opt.label}</option>
									{/each}
								</select>
							{/if}
							{#if ti.notes}
								<span class="item-notes">{ti.notes}</span>
							{/if}
							<button class="small danger" onclick={() => removeTripItem(ti.id)} style="margin-left: 4px;">×</button>
						</div>
					</div>
				{/each}
			{/if}
		</div>
	{/each}

	{#if tripItems.length === 0}
		<div class="card" style="text-align: center; color: var(--text-secondary); padding: 40px;">
			{#if trip.activity_id}
				清单为空，点击"从模板填充清单"自动添加物品
			{:else}
				清单为空，点击"添加物品"手动添加
			{/if}
		</div>
	{/if}
{:else}
	<p>加载中...</p>
{/if}

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

	.item-name {
		font-size: 15px;
	}

	.line-through {
		text-decoration: line-through;
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

	.item-notes {
		color: var(--text-secondary);
		font-style: italic;
	}

	.status-select {
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 1px 6px;
		font-size: 12px;
		cursor: pointer;
	}
</style>

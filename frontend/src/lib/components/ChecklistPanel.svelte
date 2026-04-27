<script lang="ts">
	import type { Trip, TripItem, Item, Category, Tip, Person, ItemStatus } from '$lib/types';
	import { api } from '$lib/api/client';
	import ProgressBar from './ProgressBar.svelte';
	import CategoryGroup from './CategoryGroup.svelte';
	import TripItemRow from './TripItemRow.svelte';

	let {
		trip,
		tripItems = $bindable(),
		allItems,
		categories,
		tips,
		people,
		onPopulate,
		onResync
	}: {
		trip: Trip;
		tripItems: TripItem[];
		allItems: Item[];
		categories: Category[];
		tips: Tip[];
		people: Person[];
		onPopulate: () => void;
		onResync: () => void;
	} = $props();

	let collapsed = $state<Record<number, boolean>>({});
	let showAddForm = $state(false);
	let addItemId = $state<number | null>(null);
	let addCustomName = $state('');
	let addQty = $state(1);

	// Bulk selection
	let selectable = $state(false);
	let selectedIds = $state<Set<number>>(new Set());

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

	async function toggleCheck(ti: TripItem) {
		const updated = await api.patch<TripItem>(`/trip-items/${ti.id}/check`, {
			checked: !ti.checked
		});
		tripItems = tripItems.map((t) => (t.id === updated.id ? updated : t));
	}

	async function updateField(ti: TripItem, field: string, value: unknown) {
		const updated = await api.put<TripItem>(`/trip-items/${ti.id}`, {
			...ti,
			[field]: value
		});
		tripItems = tripItems.map((t) => (t.id === updated.id ? updated : t));
	}

	async function removeTripItem(id: number) {
		await api.del(`/trip-items/${id}`);
		tripItems = tripItems.filter((t) => t.id !== id);
		selectedIds.delete(id);
	}

	async function addTripItem() {
		const body: Record<string, unknown> = { qty: addQty };
		if (addItemId) body.item_id = addItemId;
		if (addCustomName) body.custom_name = addCustomName;
		await api.post(`/trips/${trip.id}/items`, body);
		addItemId = null;
		addCustomName = '';
		addQty = 1;
		showAddForm = false;
		tripItems = await api.get<TripItem[]>(`/trips/${trip.id}/items`);
	}

	function toggleSelect(id: number) {
		const next = new Set(selectedIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selectedIds = next;
	}

	async function bulkAction(action: 'check' | 'uncheck' | 'person' | 'status', value?: unknown) {
		const ids = [...selectedIds];
		if (ids.length === 0) return;
		const body: Record<string, unknown> = { ids };
		if (action === 'check') body.checked = true;
		else if (action === 'uncheck') body.checked = false;
		else if (action === 'person') body.person_id = value;
		else if (action === 'status') body.item_status = value;

		tripItems = await api.patch<TripItem[]>(`/trips/${trip.id}/items/bulk`, body);
		selectedIds = new Set();
		selectable = false;
	}
</script>

<ProgressBar checked={totalChecked} total={totalItems} />

{#if tips.length > 0}
	<div class="card tips-card">
		{#each tips as tip}
			<div class="tip-line">⚠️ {tip.content}</div>
		{/each}
	</div>
{/if}

<div class="actions-bar">
	{#if trip.activity_id && tripItems.length === 0}
		<button class="primary" onclick={onPopulate}>从模板填充清单</button>
	{/if}
	{#if trip.activity_id && tripItems.length > 0}
		<button onclick={onResync}>同步模板</button>
	{/if}
	<button onclick={() => (showAddForm = !showAddForm)}>
		{showAddForm ? '取消' : '+ 添加物品'}
	</button>
	<button onclick={() => { selectable = !selectable; if (!selectable) selectedIds = new Set(); }}>
		{selectable ? '退出选择' : '批量操作'}
	</button>
	<button class="no-print" onclick={() => window.print()} title="打印清单">🖨️</button>
</div>

{#if selectable && selectedIds.size > 0}
	<div class="bulk-bar card">
		<span>已选 {selectedIds.size} 项</span>
		<button class="small" onclick={() => bulkAction('check')}>全部勾选</button>
		<button class="small" onclick={() => bulkAction('uncheck')}>取消勾选</button>
		{#if people.length > 0}
			<select class="small-select" onchange={(e) => {
				const val = e.currentTarget.value;
				if (val) bulkAction('person', val === 'null' ? null : Number(val));
			}}>
				<option value="">分配给...</option>
				<option value="null">未分配</option>
				{#each people as p}
					<option value={p.id}>{p.name}</option>
				{/each}
			</select>
		{/if}
	</div>
{/if}

{#if showAddForm}
	<div class="card add-form">
		<div class="add-row">
			<div class="add-field" style="flex: 1;">
				<div class="field-label">从物品库选择</div>
				<select bind:value={addItemId} style="width: 100%;">
					<option value={null}>选择物品...</option>
					{#each allItems as it}
						<option value={it.id}>{it.name} {it.brand} {it.model}</option>
					{/each}
				</select>
			</div>
			<div class="add-field">
				<div class="field-label">或自定义名称</div>
				<input bind:value={addCustomName} placeholder="自定义物品" />
			</div>
			<div class="add-field">
				<div class="field-label">数量</div>
				<input type="number" bind:value={addQty} min="1" style="width: 60px;" />
			</div>
			<button class="primary" onclick={addTripItem} disabled={!addItemId && !addCustomName} style="align-self: flex-end;">添加</button>
		</div>
	</div>
{/if}

{#each groupedItems as group}
	{@const checked = group.items.filter((t) => t.checked).length}
	<CategoryGroup
		icon={group.category.icon}
		name={group.category.name}
		{checked}
		total={group.items.length}
		collapsed={collapsed[group.category.id] ?? false}
		onToggle={() => toggleCollapse(group.category.id)}
	>
		{#each group.items as ti (ti.id)}
			<TripItemRow
				tripItem={ti}
				itemInfo={getItemInfo(ti.item_id)}
				{people}
				selected={selectedIds.has(ti.id)}
				{selectable}
				onToggleCheck={() => toggleCheck(ti)}
				onUpdateStatus={(s) => updateField(ti, 'item_status', s)}
				onUpdateQty={(q) => updateField(ti, 'qty', q)}
				onUpdateNotes={(n) => updateField(ti, 'notes', n)}
				onUpdatePerson={(id) => updateField(ti, 'person_id', id)}
				onRemove={() => removeTripItem(ti.id)}
				onToggleSelect={() => toggleSelect(ti.id)}
			/>
		{/each}
	</CategoryGroup>
{/each}

{#if tripItems.length === 0}
	<div class="card empty-state">
		{#if trip.activity_id}
			清单为空，点击"从模板填充清单"自动添加物品
		{:else}
			清单为空，点击"添加物品"手动添加
		{/if}
	</div>
{/if}

<style>
	.tips-card {
		background: #fff3cd;
		border-color: #ffc107;
		margin-bottom: 16px;
	}
	.tip-line {
		padding: 2px 0;
	}
	.actions-bar {
		display: flex;
		gap: 8px;
		margin-bottom: 16px;
		flex-wrap: wrap;
	}
	.bulk-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: #e8f0fe;
		margin-bottom: 12px;
	}
	.small-select {
		padding: 2px 6px;
		font-size: 12px;
		border: 1px solid var(--border);
		border-radius: 4px;
	}
	.add-form {
		margin-bottom: 16px;
	}
	.add-row {
		display: flex;
		gap: 10px;
		align-items: end;
	}
	.add-field {
		display: flex;
		flex-direction: column;
	}
	.field-label {
		font-size: 13px;
		color: var(--text-secondary);
		margin-bottom: 4px;
	}
	.empty-state {
		text-align: center;
		color: var(--text-secondary);
		padding: 40px;
	}
</style>

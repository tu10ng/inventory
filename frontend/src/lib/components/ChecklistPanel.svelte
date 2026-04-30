<script lang="ts">
	import type { Trip, TripItemEnriched, Item, Category, Tip, Person, ItemStatus } from '$lib/types';
	import { api } from '$lib/api/client';
	import { getDragState } from '$lib/stores/dragState.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import CategoryGroup from './CategoryGroup.svelte';
	import TripItemRow from './TripItemRow.svelte';
	import SlotRow from './SlotRow.svelte';
	import { generateTripText } from '$lib/utils/export';

	const dragState = getDragState();

	let exportLabel = $state('📋');
	let exportTimer: ReturnType<typeof setTimeout> | undefined;

	let {
		trip,
		enrichedItems = $bindable(),
		allItems,
		categories,
		tips,
		people,
		onPopulate,
		onResync,
		onReload
	}: {
		trip: Trip;
		enrichedItems: TripItemEnriched[];
		allItems: Item[];
		categories: Category[];
		tips: Tip[];
		people: Person[];
		onPopulate: () => void;
		onResync: () => void;
		onReload: () => void;
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

	function toggleCollapse(catId: number) {
		collapsed[catId] = !collapsed[catId];
	}

	const groupedItems = $derived.by(() => {
		const groups: { category: Category; items: TripItemEnriched[] }[] = [];
		const uncategorized: TripItemEnriched[] = [];
		const catMap = new Map<number, TripItemEnriched[]>();

		for (const ti of enrichedItems) {
			// Use slot's category_id if available, otherwise use item's category
			let catId: number | null = null;
			if (ti.slot) {
				catId = ti.slot.category_id;
			} else if (ti.item_id) {
				const item = getItemInfo(ti.item_id);
				catId = item?.category_id ?? null;
			}

			if (catId !== null) {
				if (!catMap.has(catId)) catMap.set(catId, []);
				catMap.get(catId)!.push(ti);
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

	const totalChecked = $derived(enrichedItems.filter((t) => t.checked).length);
	const totalItems = $derived(enrichedItems.length);

	async function toggleCheck(ti: TripItemEnriched) {
		try {
			await api.patch<unknown>(`/trip-items/${ti.id}/check`, {
				checked: !ti.checked
			});
		} catch (e) {
			console.error('切换勾选失败', e);
		}
		onReload();
	}

	async function updateField(ti: TripItemEnriched, field: string, value: unknown) {
		try {
			await api.put<unknown>(`/trip-items/${ti.id}`, {
				[field]: value
			});
		} catch (e) {
			console.error('更新字段失败', e);
		}
		onReload();
	}

	async function assignSlotItem(ti: TripItemEnriched, newItemId: number) {
		try {
			await api.put<unknown>(`/trip-items/${ti.id}`, { item_id: newItemId });
		} catch (e) {
			console.error('分配物品失败', e);
		}
		onReload();
	}

	async function clearSlotItem(ti: TripItemEnriched) {
		try {
			await api.put<unknown>(`/trip-items/${ti.id}`, { item_id: null });
		} catch (e) {
			console.error('清空物品失败', e);
		}
		onReload();
	}

	async function removeTripItem(id: number) {
		try {
			await api.del(`/trip-items/${id}`);
			selectedIds.delete(id);
		} catch (e) {
			console.error('删除物品失败', e);
		}
		onReload();
	}

	async function addTripItem() {
		if (addItemId) {
			const existing = enrichedItems.find(ti => ti.item_id === addItemId);
			if (existing) {
				const itemInfo = allItems.find(i => i.id === addItemId);
				const name = itemInfo?.name ?? '该物品';
				if (!window.confirm(`"${name}" 已在清单中，确定要再次添加吗？`)) return;
			}
		}
		const body: Record<string, unknown> = { qty: addQty };
		if (addItemId) body.item_id = addItemId;
		if (addCustomName) body.custom_name = addCustomName;
		try {
			await api.post(`/trips/${trip.id}/items`, body);
			addItemId = null;
			addCustomName = '';
			addQty = 1;
			showAddForm = false;
		} catch (e) {
			console.error('添加物品失败', e);
		}
		onReload();
	}

	function toggleSelect(id: number) {
		const next = new Set(selectedIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selectedIds = next;
	}

	async function saveTripItemAsSlot(ti: TripItemEnriched) {
		if (!trip.activity_id) return;
		try {
			await api.post(`/trip-items/${ti.id}/save-as-slot`, {});
		} catch (e) {
			console.error('保存到模板失败', e);
		}
		onReload();
	}

	async function copyExportText() {
		const text = generateTripText(trip, groupedItems, allItems, people, tips, totalChecked, totalItems);
		try {
			await navigator.clipboard.writeText(text);
			clearTimeout(exportTimer);
			exportLabel = '已复制';
			exportTimer = setTimeout(() => { exportLabel = '📋'; }, 2000);
		} catch {
			// Fallback: prompt user to copy manually
			prompt('复制以下文本:', text);
		}
	}

	async function bulkAction(action: 'check' | 'uncheck' | 'person' | 'status', value?: unknown) {
		const ids = [...selectedIds];
		if (ids.length === 0) return;
		const body: Record<string, unknown> = { ids };
		if (action === 'check') body.checked = true;
		else if (action === 'uncheck') body.checked = false;
		else if (action === 'person') body.person_id = value;
		else if (action === 'status') body.item_status = value;

		try {
			await api.patch<unknown>(`/trips/${trip.id}/items/bulk`, body);
			selectedIds = new Set();
			selectable = false;
		} catch (e) {
			console.error('批量操作失败', e);
		}
		onReload();
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
	{#if trip.activity_id && enrichedItems.length === 0}
		<button class="primary" onclick={onPopulate}>从模板填充清单</button>
	{/if}
	{#if trip.activity_id && enrichedItems.length > 0}
		<button onclick={onResync}>同步模板</button>
	{/if}
	<button onclick={() => (showAddForm = !showAddForm)}>
		{showAddForm ? '取消' : '+ 添加额外物品'}
	</button>
	<button onclick={() => { selectable = !selectable; if (!selectable) selectedIds = new Set(); }}>
		{selectable ? '退出选择' : '批量操作'}
	</button>
	<button class="no-print" onclick={() => window.print()} title="打印清单">🖨️</button>
	<button class="no-print" onclick={copyExportText} title="复制清单文本">{exportLabel}</button>
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
			{#if ti.slot_id}
				<SlotRow
					tripItem={ti}
					{people}
					selected={selectedIds.has(ti.id)}
					{selectable}
					isDragging={dragState.draggingItem !== null}
					isValidDropTarget={ti.slot_id != null && dragState.validSlotIds.has(ti.slot_id)}
					onToggleCheck={() => toggleCheck(ti)}
					onUpdateStatus={(s) => updateField(ti, 'item_status', s)}
					onUpdateQty={(q) => updateField(ti, 'qty', q)}
					onUpdateNotes={(n) => updateField(ti, 'notes', n)}
					onUpdatePerson={(id) => updateField(ti, 'person_id', id)}
					onToggleSelect={() => toggleSelect(ti.id)}
					onAssignItem={(newId) => assignSlotItem(ti, newId)}
					onClearItem={() => clearSlotItem(ti)}
				/>
			{:else}
				<TripItemRow
					tripItem={ti}
					itemInfo={getItemInfo(ti.item_id)}
					{people}
					selected={selectedIds.has(ti.id)}
					{selectable}
					canSaveAsSlot={!!trip.activity_id}
					onToggleCheck={() => toggleCheck(ti)}
					onUpdateStatus={(s) => updateField(ti, 'item_status', s)}
					onUpdateQty={(q) => updateField(ti, 'qty', q)}
					onUpdateNotes={(n) => updateField(ti, 'notes', n)}
					onUpdatePerson={(id) => updateField(ti, 'person_id', id)}
					onRemove={() => removeTripItem(ti.id)}
					onToggleSelect={() => toggleSelect(ti.id)}
					onSaveAsSlot={() => saveTripItemAsSlot(ti)}
				/>
			{/if}
		{/each}
	</CategoryGroup>
{/each}

{#if enrichedItems.length === 0}
	<div class="card empty-state">
		{#if trip.activity_id}
			清单为空，点击"从模板填充清单"自动添加物品
		{:else}
			清单为空，点击"添加额外物品"手动添加
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

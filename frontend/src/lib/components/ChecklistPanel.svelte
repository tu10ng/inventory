<script lang="ts">
	import type { Trip, TripItemEnriched, Item, Type, Tip, Person, StatusDefinition } from '$lib/types';
	import { api } from '$lib/api/client';
	import { getDragState } from '$lib/stores/dragState.svelte';
	import { getRootTypeId, getRootTypeName } from '$lib/utils/columns';
	import ProgressBar from './ProgressBar.svelte';
	import CategoryGroup from './CategoryGroup.svelte';
	import TripItemRow from './TripItemRow.svelte';
	import SlotRow from './SlotRow.svelte';
	import ChecklistAddForm from './ChecklistAddForm.svelte';
	import BulkActionBar from './BulkActionBar.svelte';
	import { generateTripText } from '$lib/utils/export';

	const dragState = getDragState();

	let exportLabel = $state('📋');
	let exportTimer: ReturnType<typeof setTimeout> | undefined;
	let errorMessage = $state<string | null>(null);
	let errorTimer: ReturnType<typeof setTimeout> | undefined;

	function showError(msg: string) {
		errorMessage = msg;
		clearTimeout(errorTimer);
		errorTimer = setTimeout(() => { errorMessage = null; }, 5000);
	}

	let panelDropOver = $state(false);

	// Always reset when drag ends globally (covers stopPropagation in SlotRow, confirm dialogs, etc.)
	$effect(() => {
		if (dragState.draggingItem === null) {
			panelDropOver = false;
		}
	});

	function handlePanelDragOver(e: DragEvent) {
		e.preventDefault();
		e.dataTransfer!.dropEffect = 'copy';
		panelDropOver = true;
	}

	function handlePanelDragLeave(e: DragEvent) {
		const ct = e.currentTarget as HTMLElement;
		if (ct === e.target || !ct.contains(e.relatedTarget as Node)) {
			panelDropOver = false;
		}
	}

	function handlePanelDrop(e: DragEvent) {
		e.preventDefault();
		panelDropOver = false;
		try {
			const data = JSON.parse(e.dataTransfer!.getData('application/json'));
			if (data.itemId) addItemByDrop(data.itemId);
		} catch { /* ignore bad data */ }
	}

	async function addItemByDrop(itemId: number) {
		const existing = enrichedItems.find(ti => ti.item_id === itemId);
		if (existing) {
			const itemInfo = allItems.find(i => i.id === itemId);
			const name = String(itemInfo?.attrs?.name ?? '该物品');
			if (!window.confirm(`"${name}" 已在清单中，确定要再次添加吗？`)) return;
		}
		const itemInfo = allItems.find(i => i.id === itemId);
		const qty = Number(itemInfo?.attrs?.default_qty ?? 1);
		try {
			await api.post(`/trips/${trip.id}/items`, { item_id: itemId, qty });
		} catch (e) {
			console.error('添加物品失败', e);
			showError('操作失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
		onReload();
	}

	let {
		trip,
		enrichedItems = $bindable(),
		allItems,
		types,
		tips,
		people,
		statusDefs = [],
		onPopulate,
		onResync,
		onReload
	}: {
		trip: Trip;
		enrichedItems: TripItemEnriched[];
		allItems: Item[];
		types: Type[];
		tips: Tip[];
		people: Person[];
		statusDefs?: StatusDefinition[];
		onPopulate: () => void;
		onResync: () => void;
		onReload: () => void;
	} = $props();

	let collapsed = $state<Record<number, boolean>>({});
	let showAddForm = $state(false);

	// Bulk selection
	let selectable = $state(false);
	let selectedIds = $state<Set<number>>(new Set());

	function getItemInfo(itemId: number | null) {
		if (!itemId) return null;
		return allItems.find((i) => i.id === itemId) ?? null;
	}

	function toggleCollapse(rootTypeId: number) {
		collapsed[rootTypeId] = !collapsed[rootTypeId];
	}

	const groupedItems = $derived.by(() => {
		const groups: { rootTypeId: number; rootTypeName: string; items: TripItemEnriched[] }[] = [];
		const catMap = new Map<number, TripItemEnriched[]>();

		for (const ti of enrichedItems) {
			let rootTypeId: number | null = null;
			if (ti.item_id) {
				const item = getItemInfo(ti.item_id);
				rootTypeId = item ? getRootTypeId(item.type_id, types) : null;
			}

			if (rootTypeId !== null && rootTypeId !== undefined) {
				if (!catMap.has(rootTypeId)) catMap.set(rootTypeId, []);
				catMap.get(rootTypeId)!.push(ti);
			} else {
				// Uncategorized
				if (!catMap.has(-1)) catMap.set(-1, []);
				catMap.get(-1)!.push(ti);
			}
		}

		const rootTypes = types.filter(t => t.parent_id === null).sort((a, b) => a.sort_order - b.sort_order);
		for (const rt of rootTypes) {
			const items = catMap.get(rt.id);
			if (items && items.length > 0) {
				groups.push({ rootTypeId: rt.id, rootTypeName: rt.name, items });
			}
		}

		const uncategorized = catMap.get(-1);
		if (uncategorized && uncategorized.length > 0) {
			groups.push({ rootTypeId: -1, rootTypeName: '其他', items: uncategorized });
		}

		return groups;
	});

	const totalChecked = $derived(enrichedItems.filter((t) => t.checked).length);
	const totalItems = $derived(enrichedItems.length);

	async function toggleCheck(ti: TripItemEnriched) {
		const prev = ti.checked;
		const idx = enrichedItems.findIndex(i => i.id === ti.id);
		if (idx >= 0) enrichedItems[idx] = { ...enrichedItems[idx], checked: !prev };
		try {
			await api.patch<unknown>(`/trip-items/${ti.id}/check`, { checked: !prev });
		} catch (e) {
			// Revert on failure
			if (idx >= 0) enrichedItems[idx] = { ...enrichedItems[idx], checked: prev };
			console.error('切换勾选失败', e);
		}
	}

	async function updateField(ti: TripItemEnriched, field: string, value: unknown) {
		const idx = enrichedItems.findIndex(i => i.id === ti.id);
		const prev = idx >= 0 ? (enrichedItems[idx] as unknown as Record<string, unknown>)[field] : undefined;
		if (idx >= 0) enrichedItems[idx] = { ...enrichedItems[idx], [field]: value };
		try {
			await api.put<unknown>(`/trip-items/${ti.id}`, { [field]: value });
		} catch (e) {
			// Revert on failure
			if (idx >= 0 && prev !== undefined) enrichedItems[idx] = { ...enrichedItems[idx], [field]: prev };
			console.error('更新字段失败', e);
		}
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

	async function handleAddItem(addItemId: number | null, addCustomName: string, addQty: number) {
		if (addItemId) {
			const existing = enrichedItems.find(ti => ti.item_id === addItemId);
			if (existing) {
				const itemInfo = allItems.find(i => i.id === addItemId);
				const name = String(itemInfo?.attrs?.name ?? '该物品');
				if (!window.confirm(`"${name}" 已在清单中，确定要再次添加吗？`)) return;
			}
		}
		const body: Record<string, unknown> = { qty: addQty };
		if (addItemId) body.item_id = addItemId;
		if (addCustomName) body.custom_name = addCustomName;
		try {
			await api.post(`/trips/${trip.id}/items`, body);
			showAddForm = false;
		} catch (e) {
			alert((e as Error).message);
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
		const text = generateTripText(
			trip,
			groupedItems.map(g => ({ rootTypeName: g.rootTypeName, items: g.items })),
			allItems,
			types,
			people,
			tips,
			totalChecked,
			totalItems
		);
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

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="checklist-drop-target"
	class:drag-over={panelDropOver}
	ondragover={handlePanelDragOver}
	ondragleave={handlePanelDragLeave}
	ondrop={handlePanelDrop}
>
{#if errorMessage}
	<div class="error-banner" role="alert">
		{errorMessage}
		<button onclick={() => { errorMessage = null; clearTimeout(errorTimer); }}>✕</button>
	</div>
{/if}
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
	<BulkActionBar
		selectedCount={selectedIds.size}
		actions={[
			{ label: '全部勾选', action: () => bulkAction('check') },
			{ label: '取消勾选', action: () => bulkAction('uncheck') },
			...people.length > 0 ? [{
				label: '分配给...',
				action: async () => {
					const idStr = prompt('输入人员 ID (留空=未分配):');
					if (idStr !== null) {
						await bulkAction('person', idStr === '' ? null : Number(idStr));
					}
				}
			}] : []
		]}
	/>
{/if}

{#if showAddForm}
	<ChecklistAddForm
		{allItems}
		onAdd={handleAddItem}
	/>
{/if}

{#each groupedItems as group}
	{@const checked = group.items.filter((t) => t.checked).length}
	<CategoryGroup
		icon=""
		name={group.rootTypeName}
		{checked}
		total={group.items.length}
		collapsed={collapsed[group.rootTypeId] ?? false}
		onToggle={() => toggleCollapse(group.rootTypeId)}
	>
		{#each group.items as ti (ti.id)}
			{#if ti.slot_id}
				<SlotRow
					tripItem={ti}
					{people}
					{statusDefs}
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
					{statusDefs}
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
			清单为空，点击"添加额外物品"或从右侧拖拽物品添加
		{/if}
	</div>
{/if}
</div>

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
	.empty-state {
		text-align: center;
		color: var(--text-secondary);
		padding: 40px;
	}
	.checklist-drop-target {
		border-radius: 8px;
		transition: outline 0.15s, background 0.15s;
	}
	.checklist-drop-target.drag-over {
		outline: 2px dashed var(--primary);
		outline-offset: -2px;
		background: rgba(59, 130, 246, 0.04);
	}
	.error-banner {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 14px;
		background: #fdf0f0;
		border: 1px solid var(--danger);
		border-radius: 8px;
		color: var(--danger);
		font-size: 13px;
		margin-bottom: 12px;
	}
	.error-banner button {
		background: none;
		border: none;
		color: var(--danger);
		font-size: 16px;
		cursor: pointer;
		padding: 0 2px;
		line-height: 1;
	}
</style>

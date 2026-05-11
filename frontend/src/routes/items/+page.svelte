<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category, Tag, ItemUsageCount, AiParsedItem, AttributeDefinition } from '$lib/types';
	import SearchFilter from '$lib/components/SearchFilter.svelte';
	import ColumnPicker from '$lib/components/ColumnPicker.svelte';
	import ItemListTable from '$lib/components/ItemListTable.svelte';
	import PanelContainer from '$lib/components/PanelContainer.svelte';
	import ItemDetailPanel from '$lib/components/ItemDetailPanel.svelte';
	import ItemForm from '$lib/components/ItemForm.svelte';
	import AiAddModal from '$lib/components/AiAddModal.svelte';
	import AiOrganizeModal from '$lib/components/AiOrganizeModal.svelte';
	import ImportModal from '$lib/components/ImportModal.svelte';
	import OrderImportModal from '$lib/components/OrderImportModal.svelte';
	import { loadAllColumns, getAllColumns, loadVisibleColumns } from '$lib/utils/columns';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import { filterItems, sortItems, groupItems } from '$lib/utils/itemFilters';
	import type { ItemGroup } from '$lib/utils/itemFilters';

	let items = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tags = $state<Tag[]>([]);
	let attrDefs = $state<AttributeDefinition[]>([]);
	let usageStats = $state<Map<number, number>>(new Map());
	let allColumns = $state<ItemColumnDef[]>([]);

	let selectedItem = $state<Item | null>(null);
	let panelMode = $state<'detail' | 'create' | null>(null);

	let search = $state('');
	let filterCategoryId = $state<number | null>(null);

	let collapsedCategories = $state<Set<number>>(new Set());
	let visibleKeys = $state<string[]>(loadVisibleColumns());
	const visibleColumns = $derived(allColumns.filter(c => visibleKeys.includes(c.key)));

	let showAiModal = $state(false);
	let showOrganizeModal = $state(false);
	let showImportModal = $state(false);
	let showOcrModal = $state(false);
	let prefillAiText = $state('');

	let sortKey = $state<string | null>(null);
	let sortDir = $state<'asc' | 'desc'>('asc');
	let columnFilters = $state<Map<string, Set<string>>>(new Map());
	let groupByKey = $state<string | null>(null);

	let loading = $state(true);
	let error = $state<string | null>(null);

	async function load() {
		try {
			loading = true;
			error = null;
			const [itemsData, cats, tagsData, adefs, cols] = await Promise.all([
				api.get<Item[]>('/items'),
				api.get<Category[]>('/categories'),
				api.get<Tag[]>('/tags'),
				api.get<AttributeDefinition[]>('/attribute-definitions'),
				loadAllColumns()
			]);
			items = itemsData;
			categories = cats;
			tags = tagsData;
			attrDefs = adefs;
			allColumns = cols;
			try {
				const stats = await api.get<ItemUsageCount[]>('/item-stats');
				usageStats = new Map(stats.map((s) => [s.item_id, s.trip_count]));
			} catch {
				// stats not critical
			}
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loading = false;
		}
	}

	function selectItem(item: Item) {
		selectedItem = item;
		panelMode = 'detail';
	}

	function startCreate() {
		selectedItem = null;
		panelMode = 'create';
	}

	async function handleFieldUpdate(field: string, value: unknown) {
		if (!selectedItem) return;
		const data: Record<string, unknown> = { ...selectedItem, [field]: value };
		// When category changes, clear tag if it doesn't belong to new category
		if (field === 'category_id') {
			const currentTag = tags.find(t => t.id === selectedItem!.tag_id);
			if (currentTag && currentTag.category_id !== value) {
				data.tag_id = null;
			}
		}
		try {
			const updated = await api.put<Item>(`/items/${selectedItem.id}`, data);
			selectedItem = updated;
			// In-place update items array instead of full reload
			const needsFullReload = field === 'category_id';
			if (needsFullReload) {
				await load();
			} else {
				items = items.map(i => i.id === updated.id ? updated : i);
			}
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function handleSave(data: Record<string, unknown>) {
		try {
			const created = await api.post<Item>('/items', data);
			selectedItem = created;
			panelMode = 'detail';
			await load();
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function handleDelete() {
		if (!selectedItem) return;
		if (!confirm(`确定删除「${selectedItem.name}」？`)) return;
		try {
			await api.del(`/items/${selectedItem.id}`);
			selectedItem = null;
			panelMode = null;
			await load();
		} catch (e) {
			alert((e as Error).message);
		}
	}

	function handleCancel() {
		selectedItem = null;
		panelMode = null;
	}

	async function handleAiConfirm(aiItems: AiParsedItem[]) {
		showAiModal = false;
		prefillAiText = '';
		for (const item of aiItems) {
			const payload = {
				name: item.name,
				brand: item.brand || '',
				model: item.model || '',
				category_id: item.category_id ?? categories[0]?.id ?? 1,
				default_qty: item.default_qty || 1,
				notes: item.notes || '',
				tag_id: item.tag_id ?? null,
				attrs: item.attrs ?? {},
			};
			try {
				await api.post('/items', payload);
			} catch (e) {
				console.error('Failed to create item:', item.name, e);
			}
		}
		await load();
	}

	function toggleCategory(catId: number) {
		const next = new Set(collapsedCategories);
		if (next.has(catId)) next.delete(catId);
		else next.add(catId);
		collapsedCategories = next;
	}

	const filteredItems = $derived(filterItems(items, search, filterCategoryId, columnFilters, allColumns, tags));

	function handleSort(key: string) {
		if (sortKey === key) {
			sortDir = sortDir === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = key;
			sortDir = 'asc';
		}
	}

	function handleFilterChange(key: string, values: Set<string>) {
		const next = new Map(columnFilters);
		if (values.size === 0) next.delete(key);
		else next.set(key, values);
		columnFilters = next;
	}

	const sortedItems = $derived(sortItems(filteredItems, sortKey, sortDir, tags));

	// Group-by options: registered text-type attributes (for grouping by attribute values)
	const groupByOptions = $derived(
		allColumns.filter(c => c.key !== 'tag' && c.key !== 'brand' && c.type === 'text')
	);

	// Group-by data: pre-computed per-category groups when groupByKey is set
	const groupedData = $derived.by(() => {
		if (!groupByKey) return null;
		const map = new Map<number, { groups: ItemGroup[]; ungrouped: Item[] }>();
		// Group within each category
		for (const cat of categories) {
			const catItems = sortedItems.filter(i => i.category_id === cat.id);
			if (catItems.length > 0) {
				map.set(cat.id, groupItems(catItems, groupByKey, allColumns));
			}
		}
		return map;
	});

	const groupBy = $derived(
		groupByKey
			? { key: groupByKey, label: allColumns.find(c => c.key === groupByKey)?.label ?? groupByKey }
			: null
	);

	// Keep selectedItem in sync after reload
	$effect(() => {
		if (selectedItem) {
			const fresh = items.find(i => i.id === selectedItem!.id);
			if (fresh) selectedItem = fresh;
		}
	});

	$effect(() => { load(); });
</script>

{#if loading}
	<div class="loading-state">加载中...</div>
{:else if error}
	<div class="error-state">
		<p>{error}</p>
		<button onclick={load}>重试</button>
	</div>
{:else}
<div class="page-container">
<h1>物品库</h1>

<div class="split-layout">
	<div class="left-panel">
		<div class="toolbar">
			<div class="toolbar-row">
				<SearchFilter
					{search}
					categoryId={filterCategoryId}
					{categories}
					onSearchChange={(v) => (search = v)}
					onCategoryChange={(id) => (filterCategoryId = id)}
				/>
				<div class="group-by-select">
					<label for="group-by-select">分组</label>
					<select id="group-by-select" value={groupByKey ?? ''} onchange={(e) => (groupByKey = e.currentTarget.value || null)}>
						<option value="">无</option>
						{#each groupByOptions as col (col.key)}
							<option value={col.key}>{col.label}</option>
						{/each}
					</select>
				</div>
				<ColumnPicker columns={allColumns} bind:visibleKeys />
			</div>
			<div class="toolbar-row toolbar-actions">
				<button onclick={() => api.downloadExport('/items/export').catch(e => alert(e.message))}>导出</button>
				<button onclick={() => showImportModal = true}>导入</button>
				<button class="primary" onclick={startCreate}>+ 添加物品</button>
				<button onclick={() => showAiModal = true}>AI 添加</button>
				<button onclick={() => showOcrModal = true}>OCR 导入</button>
				<button onclick={() => showOrganizeModal = true}>AI 整理</button>
			</div>
		</div>
		<ItemListTable
			items={sortedItems}
			{categories}
			{tags}
			{visibleColumns}
			selectedItemId={selectedItem?.id ?? null}
			{collapsedCategories}
			{sortKey}
			{sortDir}
			{columnFilters}
			{groupBy}
			{groupedData}
			onSelect={selectItem}
			onToggleCategory={toggleCategory}
			onSort={handleSort}
			onFilterChange={handleFilterChange}
		/>
	</div>

	<div class="right-panel">
		<PanelContainer>
			{#if panelMode === 'detail' && selectedItem}
				<ItemDetailPanel
					item={selectedItem}
					{categories}
					{tags}
					{attrDefs}
					usageCount={usageStats.get(selectedItem.id) ?? 0}
					onUpdate={handleFieldUpdate}
					onDelete={handleDelete}
				/>
			{:else if panelMode === 'create'}
				<ItemForm
					{categories}
					{tags}
					{attrDefs}
					onSave={handleSave}
					onCancel={handleCancel}
				/>
			{:else}
				<div class="empty-panel">
					<div class="empty-icon">📋</div>
					<p>选择物品查看详情</p>
					<p class="empty-hint">或点击"添加物品"创建新物品</p>
				</div>
			{/if}
		</PanelContainer>
	</div>
</div>
</div>

{#if showAiModal}
	<AiAddModal
		{categories}
		{tags}
		{prefillAiText}
		onConfirm={handleAiConfirm}
		onClose={() => { showAiModal = false; prefillAiText = ''; }}
		onNewTags={(newTags) => {
			const existingIds = new Set(tags.map(t => t.id));
			tags = [...tags, ...newTags.filter(t => !existingIds.has(t.id))];
		}}
	/>
{/if}

{#if showOcrModal}
	<OrderImportModal
		onClose={() => showOcrModal = false}
		onOpenAiModal={(text) => { prefillAiText = text; showAiModal = true; }}
	/>
{/if}

{#if showOrganizeModal}
	<AiOrganizeModal
		{items}
		{categories}
		{tags}
		onDone={() => load()}
		onClose={() => showOrganizeModal = false}
		onNewTags={(newTags) => {
			const existingIds = new Set(tags.map(t => t.id));
			tags = [...tags, ...newTags.filter(t => !existingIds.has(t.id))];
		}}
	/>
{/if}

{#if showImportModal}
	<ImportModal
		onClose={() => showImportModal = false}
		onDone={() => load()}
	/>
{/if}
{/if}

<style>
	.page-container {
		display: flex;
		flex-direction: column;
		height: calc(100vh - 48px);
	}
	.page-container > h1 {
		flex-shrink: 0;
	}
	.toolbar {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 12px;
		flex-shrink: 0;
	}
	.toolbar-row {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.toolbar-row :global(.search-filter) {
		flex: 1;
		margin-bottom: 0;
	}
	.group-by-select {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: var(--text-secondary);
		flex-shrink: 0;
	}
	.group-by-select select {
		font-size: 12px;
		padding: 2px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
	}
	.toolbar-actions {
		justify-content: flex-end;
	}
	.split-layout {
		display: flex;
		gap: 16px;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}
	.left-panel {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		height: 100%;
	}
	.right-panel {
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		height: 100%;
	}
	.empty-panel {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 40px 20px;
		text-align: center;
		color: var(--text-secondary);
	}
	.empty-icon {
		font-size: 40px;
		margin-bottom: 12px;
	}
	.empty-hint {
		font-size: 13px;
		margin-top: 4px;
		opacity: 0.7;
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

	@media (max-width: 768px) {
		.page-container {
			height: auto;
		}
		.split-layout {
			flex-direction: column;
			overflow: visible;
		}
		.left-panel,
		.right-panel {
			overflow-y: visible;
			height: auto;
		}
		.right-panel {
			width: 100%;
		}
		.toolbar {
			position: static;
		}
	}
</style>

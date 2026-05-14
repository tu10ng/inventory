<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category, Tag, ItemUsageCount, AiParsedItem, AttributeDefinition, DisplayRule, ItemRelationEnriched, RelationType, CreateItemRelation, DisplayRuleConfig } from '$lib/types';
	import { itemName } from '$lib/types';
	import { parseDisplayRuleConfig } from '$lib/types';
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
	import ExcelImportModal from '$lib/components/ExcelImportModal.svelte';
	import BulkActionBar from '$lib/components/BulkActionBar.svelte';
	import type { BatchAttrOption } from '$lib/components/BulkActionBar.svelte';
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

	// Batch operations — always active
	let selectedItemIds = $state<Set<number>>(new Set());

	let search = $state('');
	let filterCategoryId = $state<number | null>(null);

	let collapsedCategories = $state<Set<number>>(new Set());
	let visibleKeys = $state<string[]>(loadVisibleColumns());
	const visibleColumns = $derived(allColumns.filter(c => visibleKeys.includes(c.key)));

	let showAiModal = $state(false);
	let showOrganizeModal = $state(false);
	let showImportModal = $state(false);
	let showOcrModal = $state(false);
	let showExcelModal = $state(false);
	let prefillAiText = $state('');

	let sortKey = $state<string | null>(null);
	let sortDir = $state<'asc' | 'desc'>('asc');
	let columnFilters = $state<Map<string, Set<string>>>(new Map());
	let groupByKey = $state<string | null>(localStorage.getItem('inventory-group-by') ?? null);
	let displayRules = $state<DisplayRule[]>([]);
	let selectedRuleId = $state<number | null>(null);
	let ruleConfig = $state<DisplayRuleConfig | null>(null);
	let itemRelations = $state<ItemRelationEnriched[]>([]);
	let relationTypes = $state<RelationType[]>([]);

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
			try {
				displayRules = await api.get<DisplayRule[]>('/display-rules');
			} catch {
				// rules not critical
			}
			try {
				relationTypes = await api.get<RelationType[]>('/relation-types');
			} catch {
				// relation types not critical
			}
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loading = false;
		}
	}

	async function selectItem(item: Item) {
		selectedItem = item;
		panelMode = 'detail';
		// Load relations for this item
		try {
			itemRelations = await api.get<ItemRelationEnriched[]>(`/items/${item.id}/relations`);
		} catch {
			itemRelations = [];
		}
	}

	function startCreate() {
		selectedItem = null;
		panelMode = 'create';
	}

	async function handleFieldUpdate(field: string, value: unknown) {
		if (!selectedItem) return;
		const isTopLevel = field === 'category_id' || field === 'tag_id';
		let data: Record<string, unknown>;
		if (isTopLevel) {
			data = { [field]: value };
			// When category changes, clear tag if it doesn't belong to new category
			if (field === 'category_id') {
				const currentTag = tags.find(t => t.id === selectedItem!.tag_id);
				if (currentTag && currentTag.category_id !== value) {
					data.tag_id = null;
				}
			}
		} else if (field === 'attrs') {
			// value is already the complete attrs object (from updateAttr)
			data = { attrs: value as Record<string, unknown> };
		} else {
			// Update a single field within attrs
			const newAttrs = { ...selectedItem.attrs, [field]: value };
			data = { attrs: newAttrs };
		}
		try {
			const updated = await api.put<Item>(`/items/${selectedItem.id}`, data);
			selectedItem = updated;
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
			items = [...items, created];
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function handleDelete() {
		if (!selectedItem) return;
		if (!confirm(`确定删除「${itemName(selectedItem)}」？`)) return;
		try {
			await api.del(`/items/${selectedItem.id}`);
			const deletedId = selectedItem.id;
			selectedItem = null;
			panelMode = null;
			items = items.filter(i => i.id !== deletedId);
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function handleAddRelation(rel: CreateItemRelation) {
		if (!selectedItem) return;
		try {
			await api.post(`/items/${selectedItem.id}/relations`, rel);
			itemRelations = await api.get<ItemRelationEnriched[]>(`/items/${selectedItem.id}/relations`);
		} catch (e) {
			alert((e as Error).message);
		}
	}

	async function handleRemoveRelation(id: number) {
		try {
			await api.del(`/item-relations/${id}`);
			if (selectedItem) {
				itemRelations = await api.get<ItemRelationEnriched[]>(`/items/${selectedItem.id}/relations`);
			}
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
				category_id: item.category_id ?? categories[0]?.id ?? 1,
				tag_id: item.tag_id ?? null,
				attrs: item.attrs ?? {},
			};
			try {
				await api.post('/items', payload);
			} catch (e) {
				const name = String(item.attrs?.name ?? '?');
				console.error('Failed to create item:', name, e);
			}
		}
		await load();
	}

	function applyRule(ruleId: number | null) {
		selectedRuleId = ruleId;
		if (ruleId === null) {
			ruleConfig = null;
			return;
		}

		const rule = displayRules.find(r => r.id === ruleId);
		if (!rule) return;

		// Parse config
		const config = parseDisplayRuleConfig(rule.config);
		ruleConfig = config;

		// Set category filter (null category_id = global, keep current filter)
		filterCategoryId = rule.category_id;

		// Set grouping
		groupByKey = rule.group_by_key || null;

		// Set sorting
		sortKey = rule.sort_by_key || null;
		sortDir = rule.sort_dir === 'desc' ? 'desc' : 'asc';

		// Set visible columns
		try {
			const cols: string[] = JSON.parse(rule.visible_columns);
			if (Array.isArray(cols) && cols.length > 0) {
				visibleKeys = cols;
			}
		} catch {
			// ignore malformed JSON
		}
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

	// Persist group-by selection across sessions
	$effect(() => {
		if (groupByKey) {
			localStorage.setItem('inventory-group-by', groupByKey);
		} else {
			localStorage.removeItem('inventory-group-by');
		}
	});

	// ── Batch operations ──

	function toggleSelectItem(id: number) {
		const next = new Set(selectedItemIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		selectedItemIds = next;
	}

	async function handleBatchDelete() {
		if (!confirm(`确定删除选中的 ${selectedItemIds.size} 件物品？此操作不可撤销。`)) return;
		try {
			const idsToDelete = new Set(selectedItemIds);
			await api.post('/items/batch', { ids: [...idsToDelete], action: 'delete' });
			selectedItemIds = new Set();
			selectedItem = null;
			panelMode = null;
			items = items.filter(i => !idsToDelete.has(i.id));
		} catch (e) {
			alert('批量删除失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	async function handleBatchUpdateAttr(attrKey: string, value: unknown) {
		try {
			const idsToUpdate = new Set(selectedItemIds);
			let changes: Record<string, unknown>;

			if (attrKey === 'category_id') {
				changes = { category_id: value };
			} else if (attrKey === 'tag_id') {
				changes = { tag_id: value };
			} else {
				// attr within attrs JSON
				changes = { attrs: { [attrKey]: value } };
			}

			await api.post('/items/batch', { ids: [...idsToUpdate], action: 'update', changes });
			selectedItemIds = new Set();
			selectedItem = null;
			panelMode = null;

			// Optimistic local update
			items = items.map(i => {
				if (!idsToUpdate.has(i.id)) return i;
				if (attrKey === 'category_id') {
					return { ...i, category_id: value as number };
				}
				if (attrKey === 'tag_id') {
					return { ...i, tag_id: value as number | null };
				}
				return { ...i, attrs: { ...i.attrs, [attrKey]: value } };
			});
		} catch (e) {
			alert('批量更新失败：' + (e instanceof Error ? e.message : '未知错误'));
		}
	}

	// Build attribute options for BulkActionBar
	const batchAttrOptions: BatchAttrOption[] = $derived([
		// category_id as a select option
		{
			key: 'category_id',
			label: '分类',
			type: 'select',
			selectOptions: categories.map(c => ({ value: c.id, label: `${c.icon} ${c.name}` })),
		},
		// tag_id as a select option (with null)
		{
			key: 'tag_id',
			label: '标签',
			type: 'select',
			selectOptions: [
				{ value: null as unknown as number, label: '(无标签)' },
				...tags.map(t => ({ value: t.id, label: t.name })),
			],
		},
		// All attribute definitions as their respective types
		...attrDefs.map(ad => {
			const config = ad.config ? (() => { try { return JSON.parse(ad.config); } catch { return {}; } })() : {};
			const opt: BatchAttrOption = {
				key: ad.key,
				label: ad.label,
				type: ad.attr_type,
			};
			if (config.max !== undefined || config.suffix) {
				opt.config = {};
				if (config.max !== undefined) opt.config.max = config.max;
				if (config.suffix) opt.config.suffix = config.suffix;
			}
			if (config.options) {
				opt.config = { ...opt.config, options: config.options };
			}
			return opt;
		}),
	]);

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
				{#if displayRules.length > 0}
					<div class="rule-select">
						<label for="rule-select">规则</label>
						<select id="rule-select" value={selectedRuleId ?? ''} onchange={(e) => {
							const v = e.currentTarget.value;
							applyRule(v ? Number(v) : null);
						}}>
							<option value="">无</option>
							{#each displayRules as rule (rule.id)}
								<option value={rule.id}>{rule.name}</option>
							{/each}
						</select>
					</div>
				{/if}
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
				<button onclick={() => showExcelModal = true}>Excel 导入</button>
				<button onclick={() => showOrganizeModal = true}>AI 整理</button>
			</div>
		</div>
		<BulkActionBar
			selectedCount={selectedItemIds.size}
			attrOptions={batchAttrOptions}
			onBatchDelete={handleBatchDelete}
			onBatchUpdateAttr={handleBatchUpdateAttr}
		/>
		{#if ruleConfig?.mode === 'summary' && groupBy && groupByKey}
			{@const summaryFields = ruleConfig.summary_fields ?? []}
			<div class="summary-view">
				{#each [...(groupedData?.entries() ?? [])] as [catId, { groups, ungrouped }] (catId)}
					{@const cat = categories.find(c => c.id === catId)}
					<div class="summary-category">
						<h3 class="summary-cat-header">{cat?.icon ?? ''} {cat?.name ?? '未分类'}</h3>
						<div class="summary-grid">
							{#each groups as group (group.value)}
								<div class="summary-card card">
									<div class="summary-card-header">
										<span class="summary-group-key">{group.label || '(无)'}</span>
										<span class="summary-count">{group.items.length} 件</span>
									</div>
									<div class="summary-fields">
										{#each summaryFields as field}
											{@const values = [...new Set(group.items.map(i => String(i.attrs?.[field] ?? '')).filter(Boolean))]}
											{#if values.length > 0}
												<span class="summary-chip" title={field}>{values.join(', ')}</span>
											{/if}
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		{:else}
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
				selectedIds={selectedItemIds}
				onSelect={selectItem}
				onToggleCategory={toggleCategory}
				onSort={handleSort}
				onFilterChange={handleFilterChange}
				onToggleSelect={toggleSelectItem}
			/>
		{/if}
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
					relations={itemRelations}
					{relationTypes}
					onUpdate={handleFieldUpdate}
					onDelete={handleDelete}
					onAddRelation={handleAddRelation}
					onRemoveRelation={handleRemoveRelation}
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
		onNewAttrs={(newAttrs) => {
			const existingKeys = new Set(attrDefs.map(a => a.key));
			attrDefs = [...attrDefs, ...newAttrs.filter(a => !existingKeys.has(a.key))];
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

{#if showExcelModal}
	<ExcelImportModal
		{categories}
		{tags}
		{attrDefs}
		onDone={(created: number) => {
			if (created > 0) load();
		}}
		onClose={() => showExcelModal = false}
		onOpenAiModal={(text) => { prefillAiText = text; showAiModal = true; }}
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
	.rule-select {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: var(--text-secondary);
		flex-shrink: 0;
	}
	.rule-select select {
		font-size: 12px;
		padding: 2px 6px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
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

	.summary-view {
		overflow-y: auto;
		flex: 1;
		min-height: 0;
	}
	.summary-category {
		margin-bottom: 16px;
	}
	.summary-cat-header {
		font-size: 14px;
		font-weight: 600;
		padding: 4px 0;
		margin-bottom: 8px;
		border-bottom: 1px solid var(--border);
	}
	.summary-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 8px;
	}
	.summary-card {
		padding: 10px 12px;
	}
	.summary-card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 6px;
	}
	.summary-group-key {
		font-weight: 600;
		font-size: 13px;
	}
	.summary-count {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.summary-fields {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.summary-chip {
		font-size: 11px;
		background: var(--primary);
		color: white;
		padding: 2px 8px;
		border-radius: 10px;
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

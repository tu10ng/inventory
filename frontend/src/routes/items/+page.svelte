<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category, Tag, ItemUsageCount, AiParsedItem } from '$lib/types';
	import SearchFilter from '$lib/components/SearchFilter.svelte';
	import ColumnPicker from '$lib/components/ColumnPicker.svelte';
	import ItemListTable from '$lib/components/ItemListTable.svelte';
	import PanelContainer from '$lib/components/PanelContainer.svelte';
	import ItemDetailPanel from '$lib/components/ItemDetailPanel.svelte';
	import ItemForm from '$lib/components/ItemForm.svelte';
	import AiAddModal from '$lib/components/AiAddModal.svelte';
	import AiOrganizeModal from '$lib/components/AiOrganizeModal.svelte';
	import { ALL_COLUMNS, loadVisibleColumns } from '$lib/utils/columns';

	let items = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tags = $state<Tag[]>([]);
	let usageStats = $state<Map<number, number>>(new Map());

	let selectedItem = $state<Item | null>(null);
	let panelMode = $state<'detail' | 'create' | null>(null);

	let search = $state('');
	let filterCategoryId = $state<number | null>(null);

	let collapsedCategories = $state<Set<number>>(new Set());
	let visibleKeys = $state<string[]>(loadVisibleColumns());
	const visibleColumns = $derived(ALL_COLUMNS.filter(c => visibleKeys.includes(c.key)));

	let showAiModal = $state(false);
	let showOrganizeModal = $state(false);

	let sortKey = $state<string | null>(null);
	let sortDir = $state<'asc' | 'desc'>('asc');
	let columnFilters = $state<Map<string, Set<string>>>(new Map());

	async function load() {
		const [itemsData, cats, tagsData] = await Promise.all([
			api.get<Item[]>('/items'),
			api.get<Category[]>('/categories'),
			api.get<Tag[]>('/tags')
		]);
		items = itemsData;
		categories = cats;
		tags = tagsData;
		try {
			const stats = await api.get<ItemUsageCount[]>('/item-stats');
			usageStats = new Map(stats.map((s) => [s.item_id, s.trip_count]));
		} catch {
			// stats not critical
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
		const updated = await api.put<Item>(`/items/${selectedItem.id}`, data);
		selectedItem = updated;
		await load();
	}

	async function handleSave(data: Record<string, unknown>) {
		const created = await api.post<Item>('/items', data);
		selectedItem = created;
		panelMode = 'detail';
		await load();
	}

	async function handleDelete() {
		if (!selectedItem) return;
		if (!confirm(`确定删除「${selectedItem.name}」？`)) return;
		await api.del(`/items/${selectedItem.id}`);
		selectedItem = null;
		panelMode = null;
		await load();
	}

	function handleCancel() {
		selectedItem = null;
		panelMode = null;
	}

	async function handleAiConfirm(aiItems: AiParsedItem[]) {
		showAiModal = false;
		for (const item of aiItems) {
			const payload = {
				name: item.name,
				brand: item.brand || '',
				model: item.model || '',
				category_id: item.category_id ?? categories[0]?.id ?? 1,
				default_qty: item.default_qty || 1,
				notes: item.notes || '',
				tag_id: item.tag_id ?? null,
				warmth_rating: item.warmth_rating || 0,
				material: item.material || '',
				encumbrance: item.encumbrance || 0,
				waterproof: item.waterproof || 0,
				weight_grams: item.weight_grams || 0,
				season: item.season || '',
				body_parts: item.body_parts || '',
				env_protection: item.env_protection || 0,
				durability: item.durability || 0,
				storage_ml: item.storage_ml || 0,
				breathable: item.breathable || 0
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

	const filteredItems = $derived.by(() => {
		let list = items;
		if (search) {
			const q = search.toLowerCase();
			list = list.filter(
				(i) =>
					i.name.toLowerCase().includes(q) ||
					i.brand.toLowerCase().includes(q) ||
					i.model.toLowerCase().includes(q)
			);
		}
		if (filterCategoryId !== null) {
			list = list.filter((i) => i.category_id === filterCategoryId);
		}
		// Apply column filters
		for (const [key, vals] of columnFilters) {
			if (vals.size === 0) continue;
			const col = ALL_COLUMNS.find(c => c.key === key);
			if (!col) continue;
			list = list.filter((item) => {
				if (col.type === 'tag') {
					const t = item.tag_id ? tags.find(tg => tg.id === item.tag_id) : null;
					const display = t ? t.name : '-';
					return vals.has(display);
				} else if (col.type === 'bool') {
					const v = (item as unknown as Record<string, unknown>)[col.key] as number;
					return vals.has(v > 0 ? '1' : '0');
				} else {
					const v = (item as unknown as Record<string, unknown>)[col.key];
					return vals.has(v ? String(v) : '-');
				}
			});
		}
		return list;
	});

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

	// Sort items within each category group
	const sortedItems = $derived.by(() => {
		if (!sortKey) return filteredItems;
		const key = sortKey;
		const dir = sortDir;
		return [...filteredItems].sort((a, b) => {
			let va: unknown, vb: unknown;
			if (key === 'name') {
				va = a.name;
				vb = b.name;
			} else if (key === 'tag') {
				const ta = a.tag_id ? tags.find(t => t.id === a.tag_id) : null;
				const tb = b.tag_id ? tags.find(t => t.id === b.tag_id) : null;
				va = ta?.name ?? '';
				vb = tb?.name ?? '';
			} else {
				va = (a as unknown as Record<string, unknown>)[key];
				vb = (b as unknown as Record<string, unknown>)[key];
			}
			// Nullish values go last
			if (va == null && vb == null) return 0;
			if (va == null) return 1;
			if (vb == null) return -1;
			let cmp: number;
			if (typeof va === 'string' && typeof vb === 'string') {
				cmp = va.localeCompare(vb, 'zh');
			} else {
				cmp = Number(va) - Number(vb);
			}
			return dir === 'asc' ? cmp : -cmp;
		});
	});

	// Keep selectedItem in sync after reload
	$effect(() => {
		if (selectedItem) {
			const fresh = items.find(i => i.id === selectedItem!.id);
			if (fresh) selectedItem = fresh;
		}
	});

	$effect(() => { load(); });
</script>

<div class="page-container">
<h1>物品库</h1>

<div class="split-layout">
	<div class="left-panel">
		<div class="toolbar">
			<SearchFilter
				{search}
				categoryId={filterCategoryId}
				{categories}
				onSearchChange={(v) => (search = v)}
				onCategoryChange={(id) => (filterCategoryId = id)}
			/>
			<ColumnPicker bind:visibleKeys />
			<button class="primary" onclick={startCreate}>+ 添加物品</button>
			<button onclick={() => showAiModal = true}>AI 添加</button>
		<button onclick={() => showOrganizeModal = true}>AI 整理</button>
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
					usageCount={usageStats.get(selectedItem.id) ?? 0}
					onUpdate={handleFieldUpdate}
					onDelete={handleDelete}
				/>
			{:else if panelMode === 'create'}
				<ItemForm
					{categories}
					{tags}
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
		onConfirm={handleAiConfirm}
		onClose={() => showAiModal = false}
		onNewTags={(newTags) => {
			const existingIds = new Set(tags.map(t => t.id));
			tags = [...tags, ...newTags.filter(t => !existingIds.has(t.id))];
		}}
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
		gap: 8px;
		align-items: flex-start;
		margin-bottom: 12px;
		flex-shrink: 0;
		padding-bottom: 4px;
	}
	.toolbar :global(.search-filter) {
		flex: 1;
		margin-bottom: 0;
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

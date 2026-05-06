<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category, Tag, ItemUsageCount } from '$lib/types';
	import SearchFilter from '$lib/components/SearchFilter.svelte';
	import ItemListTable from '$lib/components/ItemListTable.svelte';
	import PanelContainer from '$lib/components/PanelContainer.svelte';
	import ItemDetailPanel from '$lib/components/ItemDetailPanel.svelte';
	import ItemForm from '$lib/components/ItemForm.svelte';

	let items = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tags = $state<Tag[]>([]);
	let usageStats = $state<Map<number, number>>(new Map());

	let selectedItem = $state<Item | null>(null);
	let panelMode = $state<'detail' | 'create' | null>(null);

	let search = $state('');
	let filterCategoryId = $state<number | null>(null);

	let collapsedCategories = $state<Set<number>>(new Set());

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
		return list;
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

<div class="page-header">
	<h1>物品库</h1>
	<div class="header-actions">
		<button class="primary" onclick={startCreate}>+ 添加物品</button>
	</div>
</div>

<div class="toolbar">
	<SearchFilter
		{search}
		categoryId={filterCategoryId}
		{categories}
		onSearchChange={(v) => (search = v)}
		onCategoryChange={(id) => (filterCategoryId = id)}
	/>
</div>

<div class="split-layout">
	<div class="left-panel">
		<ItemListTable
			items={filteredItems}
			{categories}
			{tags}
			selectedItemId={selectedItem?.id ?? null}
			{collapsedCategories}
			onSelect={selectItem}
			onToggleCategory={toggleCategory}
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

<style>
	.page-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
	}
	.header-actions {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.toolbar {
		display: flex;
		gap: 8px;
		align-items: flex-start;
		margin-bottom: 12px;
	}
	.toolbar :global(.search-filter) {
		flex: 1;
		margin-bottom: 0;
	}
	.split-layout {
		display: flex;
		gap: 16px;
		align-items: flex-start;
	}
	.left-panel {
		flex: 1;
		min-width: 0;
	}
	.right-panel {
		flex: 1;
		min-width: 0;
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
		.split-layout {
			flex-direction: column;
		}
		.right-panel {
			width: 100%;
		}
		.right-panel :global(.panel-container) {
			width: 100%;
			position: static;
			max-height: none;
		}
	}
</style>

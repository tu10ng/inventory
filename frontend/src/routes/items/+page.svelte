<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category, ItemUsageCount } from '$lib/types';
	import SearchFilter from '$lib/components/SearchFilter.svelte';
	import ItemCard from '$lib/components/ItemCard.svelte';

	let items = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let usageStats = $state<Map<number, number>>(new Map());
	let showForm = $state(false);
	let editingId = $state<number | null>(null);
	let form = $state({ name: '', brand: '', model: '', category_id: 0, default_qty: 1, notes: '' });
	let viewMode = $state<'list' | 'grid'>('list');
	let search = $state('');
	let filterCategoryId = $state<number | null>(null);

	async function load() {
		const [itemsData, cats] = await Promise.all([
			api.get<Item[]>('/items'),
			api.get<Category[]>('/categories')
		]);
		items = itemsData;
		categories = cats;
		if (form.category_id === 0 && categories.length > 0) {
			form.category_id = categories[0].id;
		}
		// Load stats separately so failure doesn't block the page
		try {
			const stats = await api.get<ItemUsageCount[]>('/item-stats');
			usageStats = new Map(stats.map((s) => [s.item_id, s.trip_count]));
		} catch {
			// stats not critical
		}
	}

	function resetForm() {
		form = { name: '', brand: '', model: '', category_id: categories[0]?.id ?? 0, default_qty: 1, notes: '' };
		editingId = null;
		showForm = false;
	}

	function startEdit(item: Item) {
		form = { name: item.name, brand: item.brand, model: item.model, category_id: item.category_id, default_qty: item.default_qty, notes: item.notes };
		editingId = item.id;
		showForm = true;
	}

	async function save() {
		if (editingId) {
			await api.put(`/items/${editingId}`, form);
		} else {
			await api.post('/items', form);
		}
		resetForm();
		await load();
	}

	async function remove(id: number) {
		await api.del(`/items/${id}`);
		await load();
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

	function getCategoryIcon(catId: number): string {
		return categories.find((c) => c.id === catId)?.icon ?? '📦';
	}

	$effect(() => { load(); });
</script>

<div class="page-header">
	<h1>物品库</h1>
	<div class="header-actions">
		<div class="view-toggle">
			<button class="small" class:active={viewMode === 'list'} onclick={() => (viewMode = 'list')}>列表</button>
			<button class="small" class:active={viewMode === 'grid'} onclick={() => (viewMode = 'grid')}>网格</button>
		</div>
		<button class="primary" onclick={() => { if (showForm) resetForm(); else showForm = true; }}>
			{showForm ? '取消' : '+ 添加物品'}
		</button>
	</div>
</div>

<SearchFilter
	{search}
	categoryId={filterCategoryId}
	{categories}
	onSearchChange={(v) => (search = v)}
	onCategoryChange={(id) => (filterCategoryId = id)}
/>

{#if showForm}
	<div class="card">
		<div style="display: flex; flex-direction: column; gap: 10px;">
			<div style="display: flex; gap: 10px;">
				<input bind:value={form.name} placeholder="物品名称" style="flex: 1;" />
				<select bind:value={form.category_id}>
					{#each categories as c}
						<option value={c.id}>{c.icon} {c.name}</option>
					{/each}
				</select>
			</div>
			<div style="display: flex; gap: 10px;">
				<input bind:value={form.brand} placeholder="品牌" style="flex: 1;" />
				<input bind:value={form.model} placeholder="型号" style="flex: 1;" />
				<input type="number" bind:value={form.default_qty} min="1" style="width: 60px;" />
			</div>
			<input bind:value={form.notes} placeholder="备注" />
			<button class="primary" onclick={save} disabled={!form.name}>
				{editingId ? '更新' : '添加'}
			</button>
		</div>
	</div>
{/if}

{#if viewMode === 'grid'}
	{#each categories as cat}
		{@const catItems = filteredItems.filter((i) => i.category_id === cat.id)}
		{#if catItems.length > 0}
			<div class="grid-section">
				<h3 class="grid-section-title">{cat.icon} {cat.name} ({catItems.length})</h3>
				<div class="items-grid">
					{#each catItems as item (item.id)}
						<div class="grid-card-wrapper">
							<ItemCard
								name={item.name}
								brand={item.brand}
								model={item.model}
								categoryIcon={cat.icon}
								qty={item.default_qty}
								onclick={() => startEdit(item)}
							/>
							{#if usageStats.get(item.id)}
								<div class="usage-badge" title="被 {usageStats.get(item.id)} 个行程使用">
									{usageStats.get(item.id)} 行程
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/each}
{:else}
	{#each categories as cat}
		{@const catItems = filteredItems.filter((i) => i.category_id === cat.id)}
		{#if catItems.length > 0}
			<div style="margin-bottom: 16px;">
				<h3 style="margin-bottom: 8px;">{cat.icon} {cat.name} ({catItems.length})</h3>
				{#each catItems as item (item.id)}
					<div class="card list-item">
						<div class="list-item-info">
							<strong>{item.name}</strong>
							{#if item.brand || item.model}
								<span class="list-detail">{item.brand} {item.model}</span>
							{/if}
							{#if item.default_qty > 1}
								<span class="list-detail"> x{item.default_qty}</span>
							{/if}
							{#if item.notes}
								<span class="list-notes">{item.notes}</span>
							{/if}
							{#if usageStats.get(item.id)}
								<span class="usage-inline">{usageStats.get(item.id)} 行程使用</span>
							{/if}
						</div>
						<div style="display: flex; gap: 6px;">
							<button class="small" onclick={() => startEdit(item)}>编辑</button>
							<button class="small danger" onclick={() => remove(item.id)}>删除</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{/each}
{/if}

{#if filteredItems.length === 0}
	<div class="card" style="text-align: center; color: var(--text-secondary); padding: 40px;">
		{#if items.length === 0}
			物品库为空，点击上方按钮添加物品
		{:else}
			没有匹配的物品
		{/if}
	</div>
{/if}

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
	.view-toggle {
		display: flex;
		border: 1px solid var(--border);
		border-radius: 4px;
		overflow: hidden;
	}
	.view-toggle button {
		border: none;
		border-radius: 0;
		padding: 4px 10px;
	}
	.view-toggle button.active {
		background: var(--primary);
		color: white;
	}
	.grid-section {
		margin-bottom: 16px;
	}
	.grid-section-title {
		margin-bottom: 8px;
	}
	.items-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
		gap: 10px;
	}
	.grid-card-wrapper {
		position: relative;
	}
	.usage-badge {
		position: absolute;
		bottom: 4px;
		right: 4px;
		background: var(--primary);
		color: white;
		font-size: 10px;
		padding: 1px 6px;
		border-radius: 8px;
	}
	.list-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px 16px;
	}
	.list-item-info {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.list-detail {
		color: var(--text-secondary);
	}
	.list-notes {
		color: var(--text-secondary);
		font-size: 13px;
	}
	.usage-inline {
		font-size: 11px;
		background: #e8f0fe;
		color: var(--primary);
		padding: 0 6px;
		border-radius: 8px;
	}
</style>

<script lang="ts">
	import { onMount } from 'svelte';
	import ItemListTable from '$lib/components/ItemListTable.svelte';
	import { api } from '$lib/api/client';
	import type { Type, TypeTreeNode, Category, Item, AttributeDefinition } from '$lib/types';
	import { buildTypePath, getDescendantTypeIds } from '$lib/utils/columns';

	let types = $state<Type[]>([]);
	let categories = $state<Category[]>([]);
	let items = $state<Item[]>([]);
	let attrDefs = $state<AttributeDefinition[]>([]);

	let selectedTypeId = $state<number | null>(null);
	let expandedCategories = $state<Set<number>>(new Set());
	let loading = $state(true);
	let error = $state('');

	// Build tree from flat types
	function buildTree(catTypes: Type[]): TypeTreeNode[] {
		const map = new Map<number, TypeTreeNode>();
		const roots: TypeTreeNode[] = [];
		for (const t of catTypes) {
			map.set(t.id, { ...t, children: [] });
		}
		for (const t of catTypes) {
			const node = map.get(t.id)!;
			if (t.parent_id && map.has(t.parent_id)) {
				map.get(t.parent_id)!.children.push(node);
			} else {
				roots.push(node);
			}
		}
		return roots;
	}

	// Recursive item count
	function countItems(typeId: number): number {
		const descendantIds = getDescendantTypeIds(typeId, types);
		return items.filter(i => descendantIds.has(i.type_id ?? 0)).length;
	}

	// Items for selected type (including descendants)
	let filteredItems = $derived.by(() => {
		if (selectedTypeId == null) return [] as Item[];
		const descendantIds = getDescendantTypeIds(selectedTypeId, types);
		return items.filter(i => descendantIds.has(i.type_id ?? 0));
	});

	// Selected type breadcrumb
	let breadcrumb = $derived.by(() => {
		if (selectedTypeId == null) return '';
		const type = types.find(t => t.id === selectedTypeId);
		if (!type) return '';
		const cat = categories.find(c => c.id === type.category_id);
		const typePath = buildTypePath(selectedTypeId, types);
		return (cat ? cat.icon + ' ' + cat.name : '') + ' > ' + typePath;
	});

	function toggleCategory(catId: number) {
		if (expandedCategories.has(catId)) {
			expandedCategories.delete(catId);
		} else {
			expandedCategories.add(catId);
		}
		expandedCategories = new Set(expandedCategories);
	}

	onMount(async () => {
		try {
			const [catRes, typeRes, itemRes, attrRes] = await Promise.all([
				api.get('/categories'),
				api.get('/types'),
				api.get('/items'),
				api.get('/attribute-definitions'),
			]);
			categories = catRes as Category[];
			types = typeRes as Type[];
			items = itemRes as Item[];
			attrDefs = attrRes as AttributeDefinition[];
		} catch (e) {
			error = '加载数据失败: ' + (e as Error).message;
		} finally {
			loading = false;
		}
	});
</script>

{#if loading}
	<div class="loading">加载中...</div>
{:else if error}
	<div class="error">{error}</div>
{:else}
	<div class="types-page">
		<!-- Sidebar: Type Tree -->
		<aside class="type-tree-panel">
			<h2>类型树</h2>
			{#each categories as cat}
				{@const catTypes = types.filter(t => t.category_id === cat.id)}
				{#if catTypes.length > 0}
					{@const totalItems = items.filter(i => i.category_id === cat.id).length}
					<div class="type-category">
						<button class="cat-header" onclick={() => toggleCategory(cat.id)}>
							<span class="cat-toggle">{expandedCategories.has(cat.id) ? '▾' : '▸'}</span>
							<span class="cat-icon">{cat.icon}</span>
							<span class="cat-name">{cat.name}</span>
							<span class="cat-count">{totalItems}</span>
						</button>
						{#if expandedCategories.has(cat.id)}
							<div class="tree-nodes">
								{#each buildTree(catTypes) as node}
									{@render RecursiveTreeNode({ node, types, selectedTypeId, countItems, onSelect: (id: number) => selectedTypeId = id, depth: 0 })}
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			{/each}
		</aside>

		<!-- Main: Item List -->
		<main class="type-items-panel">
			{#if selectedTypeId == null}
				<div class="empty-state">
					<p>请选择一个类型查看物品</p>
				</div>
			{:else}
				<div class="type-items-header">
					<span class="breadcrumb">{breadcrumb}</span>
					<span class="item-count">{filteredItems.length} 件物品</span>
				</div>
				<div class="type-items-list">
					<ItemListTable
						items={filteredItems as Item[]}
						{categories}
						{types}
						visibleColumns={[{ key: 'type', label: '类型', type: 'type' } as import('$lib/utils/columns').ItemColumnDef,
							{ key: 'name', label: '名称', type: 'text' },
							{ key: 'brand', label: '品牌', type: 'text' },
							{ key: 'model', label: '型号', type: 'text' }]}
						selectable={false}
						selectedItemId={null}
						collapsedCategories={new Set()}
						onSelect={() => {}}
						onToggleCategory={() => {}}
					/>
				</div>
			{/if}
		</main>
	</div>
{/if}

<!-- Recursive TreeNode snippet -->
{#snippet RecursiveTreeNode(props: { node: TypeTreeNode; types: Type[]; selectedTypeId: number | null; countItems: (id: number) => number; onSelect: (id: number) => void; depth: number })}
	{@const { node, selectedTypeId, countItems, onSelect, depth } = props}
	{@const hasChildren = node.children.length > 0}
	{@const itemCount = countItems(node.id)}
	<div class="tree-node" style="padding-left: {depth * 1.2}em">
		{#if hasChildren}
			<span class="node-toggle">▸</span>
		{:else}
			<span class="node-toggle-spacer"></span>
		{/if}
		<button
			class="node-label"
			class:selected={selectedTypeId === node.id}
			onclick={() => onSelect(node.id)}
		>
			{node.name}
			<span class="node-count">{itemCount}</span>
		</button>
	</div>
	{#each node.children as child}
		{@render RecursiveTreeNode({ node: child, types: props.types, selectedTypeId, countItems, onSelect, depth: depth + 1 })}
	{/each}
{/snippet}

<style>
	.types-page {
		display: flex;
		height: calc(100vh - 48px);
		gap: 0;
	}
	.type-tree-panel {
		width: 280px;
		border-right: 1px solid var(--border);
		overflow-y: auto;
		padding: 1rem;
		flex-shrink: 0;
	}
	.type-tree-panel h2 {
		font-size: 1.1rem;
		margin: 0 0 1rem 0;
	}
	.type-category {
		margin-bottom: 0.5rem;
	}
	.cat-header {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		width: 100%;
		padding: 0.4rem 0.25rem;
		border: none;
		background: none;
		cursor: pointer;
		font-size: 0.9rem;
		border-radius: 4px;
	}
	.cat-header:hover {
		background: var(--hover-bg, #f0f0f0);
	}
	.cat-toggle { width: 1em; flex-shrink: 0; }
	.cat-icon { flex-shrink: 0; }
	.cat-name { flex: 1; text-align: left; }
	.cat-count {
		background: var(--pill-bg, #e0e0e0);
		border-radius: 10px;
		padding: 0 0.5rem;
		font-size: 0.75rem;
		color: var(--text-secondary);
	}
	.tree-nodes {
		margin-left: 0.5rem;
	}
	.tree-node {
		display: flex;
		align-items: center;
		gap: 0.15rem;
		padding: 0.15rem 0;
	}
	.node-toggle, .node-toggle-spacer {
		width: 1.2em;
		flex-shrink: 0;
		border: none;
		background: none;
		font-size: 0.8rem;
		color: var(--text-secondary);
	}
	.node-label {
		flex: 1;
		text-align: left;
		padding: 0.2rem 0.4rem;
		border: none;
		background: none;
		cursor: pointer;
		border-radius: 4px;
		font-size: 0.85rem;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.node-label:hover { background: var(--hover-bg, #f0f0f0); }
	.node-label.selected { background: var(--accent-bg, #e8f0fe); color: var(--accent-text, #4a90d9); }
	.node-count {
		background: var(--pill-bg, #e0e0e0);
		border-radius: 10px;
		padding: 0 0.4rem;
		font-size: 0.7rem;
		color: var(--text-secondary);
	}
	.type-items-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}
	.type-items-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1rem;
		border-bottom: 1px solid var(--border);
	}
	.breadcrumb { font-size: 0.9rem; }
	.item-count { font-size: 0.8rem; color: var(--text-secondary); }
	.type-items-list {
		flex: 1;
		overflow-y: auto;
	}
	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--text-secondary);
		font-size: 1rem;
	}
	.loading, .error {
		padding: 2rem;
		text-align: center;
	}
	.error { color: var(--danger); }

	@media (max-width: 768px) {
		.types-page {
			flex-direction: column;
		}
		.type-tree-panel {
			width: 100%;
			max-height: 40vh;
			border-right: none;
			border-bottom: 1px solid var(--border);
		}
	}
</style>

<script lang="ts">
	import { api } from '$lib/api/client';
	import type { Item, Category, Tag, AttributeDefinition, DisplayRule, DisplayRuleConfig } from '$lib/types';
	import { parseDisplayRuleConfig } from '$lib/types';
	import ItemListTable from '$lib/components/ItemListTable.svelte';
	import { loadAllColumns } from '$lib/utils/columns';
	import type { ItemColumnDef } from '$lib/utils/columns';
	import { filterItems, sortItems, groupItems } from '$lib/utils/itemFilters';
	import type { ItemGroup } from '$lib/utils/itemFilters';

	let items = $state<Item[]>([]);
	let categories = $state<Category[]>([]);
	let tags = $state<Tag[]>([]);
	let attrDefs = $state<AttributeDefinition[]>([]);
	let allColumns = $state<ItemColumnDef[]>([]);
	let displayRules = $state<DisplayRule[]>([]);

	let selectedRuleId = $state<number | null>(null);
	let ruleConfig = $state<DisplayRuleConfig | null>(null);

	let search = $state('');
	let filterCategoryId = $state<number | null>(null);
	let collapsedCategories = $state<Set<number>>(new Set());
	let visibleKeys = $state<string[]>([]);

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
				displayRules = await api.get<DisplayRule[]>('/display-rules');
			} catch {
				// rules not critical
			}
		} catch (e) {
			error = (e as Error).message;
		} finally {
			loading = false;
		}
	}

	function applyRule(ruleId: number | null) {
		selectedRuleId = ruleId;
		if (ruleId === null) {
			ruleConfig = null;
			return;
		}

		const rule = displayRules.find(r => r.id === ruleId);
		if (!rule) return;

		const config = parseDisplayRuleConfig(rule.config);
		ruleConfig = config;

		filterCategoryId = rule.category_id;
		groupByKey = rule.group_by_key || null;
		sortKey = rule.sort_by_key || null;
		sortDir = rule.sort_dir === 'desc' ? 'desc' : 'asc';

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

	const filteredItems = $derived(filterItems(items, search, filterCategoryId, columnFilters, allColumns, tags));
	const sortedItems = $derived(sortItems(filteredItems, sortKey, sortDir, tags));

	const groupBy = $derived(
		groupByKey
			? { key: groupByKey, label: allColumns.find(c => c.key === groupByKey)?.label ?? groupByKey }
			: null
	);

	const groupedData = $derived.by(() => {
		if (!groupByKey) return null;
		const map = new Map<number, { groups: ItemGroup[]; ungrouped: Item[] }>();
		for (const cat of categories) {
			const catItems = sortedItems.filter(i => i.category_id === cat.id);
			if (catItems.length > 0) {
				map.set(cat.id, groupItems(catItems, groupByKey, allColumns, tags));
			}
		}
		return map;
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
<div class="views-layout">
	<aside class="rules-sidebar">
		<h3>展示规则</h3>
		<div class="rule-list">
			{#each displayRules as rule (rule.id)}
				<button
					class="rule-item"
					class:active={selectedRuleId === rule.id}
					onclick={() => applyRule(rule.id)}
				>
					{rule.name}
				</button>
			{/each}
			{#if displayRules.length === 0}
				<p class="no-rules">暂无规则，请前往设置页创建</p>
			{/if}
		</div>
		<a href="/settings" class="manage-link">管理规则 →</a>
	</aside>

	<main class="rules-content">
		{#if !selectedRuleId}
			<div class="empty-state">
				<div class="empty-icon">🔍</div>
				<p>请选择一个展示规则</p>
				<p class="empty-hint">选择左侧规则以按规则浏览物品</p>
			</div>
		{:else if ruleConfig?.mode === 'summary' && groupBy && groupByKey}
			{@const summaryFields = ruleConfig.summary_fields ?? []}
			<div class="summary-view">
				<h2 class="rule-title">{displayRules.find(r => r.id === selectedRuleId)?.name ?? ''}</h2>
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
			<h2 class="rule-title">{displayRules.find(r => r.id === selectedRuleId)?.name ?? ''}</h2>
			<ItemListTable
				items={sortedItems}
				{categories}
				{tags}
				visibleColumns={visibleKeys.length > 0 ? allColumns.filter(c => visibleKeys.includes(c.key)) : allColumns.filter(c => ['name', 'brand', 'model', 'weight'].includes(c.key))}
				selectedItemId={null}
				{collapsedCategories}
				{sortKey}
				{sortDir}
				{columnFilters}
				{groupBy}
				{groupedData}
				selectable={false}
				onSelect={() => {}}
				onToggleCategory={toggleCategory}
				onSort={handleSort}
				onFilterChange={handleFilterChange}
			/>
		{/if}
	</main>
</div>
{/if}

<style>
	.views-layout {
		display: flex;
		height: calc(100vh - 48px);
		gap: 0;
	}

	.rules-sidebar {
		width: 240px;
		flex-shrink: 0;
		border-right: 1px solid var(--border);
		padding: 20px 16px;
		display: flex;
		flex-direction: column;
		background: var(--surface);
	}

	.rules-sidebar h3 {
		font-size: 14px;
		font-weight: 600;
		margin: 0 0 12px 0;
		color: var(--text-secondary);
	}

	.rule-list {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 2px;
		overflow-y: auto;
	}

	.rule-item {
		display: block;
		width: 100%;
		text-align: left;
		padding: 8px 12px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--text);
		font-size: 13px;
		cursor: pointer;
		transition: background 0.15s;
	}

	.rule-item:hover {
		background: var(--bg);
	}

	.rule-item.active {
		background: var(--primary);
		color: white;
	}

	.no-rules {
		font-size: 12px;
		color: var(--text-secondary);
		padding: 8px 12px;
	}

	.manage-link {
		display: block;
		margin-top: 12px;
		padding: 8px 12px;
		font-size: 12px;
		color: var(--text-secondary);
		text-decoration: none;
		border-top: 1px solid var(--border);
	}

	.manage-link:hover {
		color: var(--primary);
	}

	.rules-content {
		flex: 1;
		min-width: 0;
		padding: 20px 24px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
	}

	.rule-title {
		font-size: 18px;
		font-weight: 600;
		margin: 0 0 12px 0;
		flex-shrink: 0;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--text-secondary);
	}

	.empty-icon {
		font-size: 48px;
		margin-bottom: 16px;
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
		.views-layout {
			flex-direction: column;
			height: auto;
		}

		.rules-sidebar {
			width: 100%;
			border-right: none;
			border-bottom: 1px solid var(--border);
		}

		.rule-list {
			flex-direction: row;
			flex-wrap: wrap;
			gap: 4px;
		}

		.rule-item {
			width: auto;
		}
	}
</style>

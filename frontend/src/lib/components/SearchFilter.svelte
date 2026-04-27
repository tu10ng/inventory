<script lang="ts">
	import type { Category } from '$lib/types';

	let { search, categoryId, categories, onSearchChange, onCategoryChange, dark = false }: {
		search: string;
		categoryId: number | null;
		categories: Category[];
		onSearchChange: (val: string) => void;
		onCategoryChange: (id: number | null) => void;
		dark?: boolean;
	} = $props();
</script>

<div class="search-filter" class:dark>
	<input
		class="search-input"
		type="text"
		value={search}
		oninput={(e) => onSearchChange(e.currentTarget.value)}
		placeholder="搜索物品..."
	/>
	<select
		class="category-select"
		value={categoryId ?? ''}
		onchange={(e) => {
			const val = e.currentTarget.value;
			onCategoryChange(val ? Number(val) : null);
		}}
	>
		<option value="">全部分类</option>
		{#each categories as cat}
			<option value={cat.id}>{cat.icon} {cat.name}</option>
		{/each}
	</select>
</div>

<style>
	.search-filter {
		display: flex;
		gap: 8px;
		margin-bottom: 12px;
	}
	.search-input {
		flex: 1;
		padding: 6px 10px;
		border: 1px solid var(--border);
		border-radius: 4px;
		font-size: 14px;
	}
	.category-select {
		padding: 6px 10px;
		border: 1px solid var(--border);
		border-radius: 4px;
		font-size: 14px;
	}
	.dark .search-input,
	.dark .category-select {
		background: var(--inventory-surface);
		border-color: var(--inventory-border);
		color: var(--inventory-text);
	}
	.dark .search-input::placeholder {
		color: var(--inventory-text-secondary);
	}
</style>

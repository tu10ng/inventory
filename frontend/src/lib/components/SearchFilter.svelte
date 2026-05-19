<script lang="ts">
	import type { Type } from '$lib/types';

	let { search, rootTypeId, rootTypes, onSearchChange, onRootTypeChange }: {
		search: string;
		rootTypeId: number | null;
		rootTypes: Type[];
		onSearchChange: (val: string) => void;
		onRootTypeChange: (id: number | null) => void;
	} = $props();
</script>

<div class="search-filter">
	<input
		class="search-input"
		type="text"
		value={search}
		oninput={(e) => onSearchChange(e.currentTarget.value)}
		placeholder="搜索物品..."
	/>
	<select
		class="category-select"
		value={rootTypeId ?? ''}
		onchange={(e) => {
			const val = e.currentTarget.value;
			onRootTypeChange(val ? Number(val) : null);
		}}
	>
		<option value="">全部类型</option>
		{#each rootTypes as rt}
			<option value={rt.id}>{rt.name}</option>
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
		border-radius: 6px;
		font-size: 14px;
	}
	.category-select {
		padding: 6px 10px;
		border: 1px solid var(--border);
		border-radius: 6px;
		font-size: 14px;
	}
</style>

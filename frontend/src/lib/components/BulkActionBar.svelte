<script lang="ts">
	import type { Person } from '$lib/types';

	let {
		selectedCount,
		people,
		onCheck,
		onUncheck,
		onAssignPerson
	}: {
		selectedCount: number;
		people: Person[];
		onCheck: () => void;
		onUncheck: () => void;
		onAssignPerson: (personId: number | null) => void;
	} = $props();
</script>

<div class="bulk-bar card">
	<span>已选 {selectedCount} 项</span>
	<button class="small" onclick={onCheck}>全部勾选</button>
	<button class="small" onclick={onUncheck}>取消勾选</button>
	{#if people.length > 0}
		<select class="small-select" onchange={(e) => {
			const val = e.currentTarget.value;
			if (val) onAssignPerson(val === 'null' ? null : Number(val));
		}}>
			<option value="">分配给...</option>
			<option value="null">未分配</option>
			{#each people as p}
				<option value={p.id}>{p.name}</option>
			{/each}
		</select>
	{/if}
</div>

<style>
	.bulk-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		background: #e8f0fe;
		margin-bottom: 12px;
	}
	.small-select {
		padding: 2px 6px;
		font-size: 12px;
		border: 1px solid var(--border);
		border-radius: 4px;
	}
</style>

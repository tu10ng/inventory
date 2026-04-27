<script lang="ts">
	import type { Person } from '$lib/types';

	let { personId, people, onchange }: {
		personId: number | null;
		people: Person[];
		onchange: (id: number | null) => void;
	} = $props();
</script>

<select
	class="person-select"
	value={personId ?? ''}
	onchange={(e) => {
		const val = e.currentTarget.value;
		onchange(val ? Number(val) : null);
	}}
	onclick={(e) => e.stopPropagation()}
>
	<option value="">未分配</option>
	{#each people as p}
		<option value={p.id}>{p.name}</option>
	{/each}
</select>

<style>
	.person-select {
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 1px 6px;
		font-size: 12px;
		cursor: pointer;
		background: var(--surface);
		max-width: 80px;
	}
</style>

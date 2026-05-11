<script lang="ts">
	import type { StatusDefinition } from '$lib/types';

	let { status, statusDefs = [], onchange }: {
		status: string;
		statusDefs?: StatusDefinition[];
		onchange: (s: string) => void;
	} = $props();

	const options = $derived(
		statusDefs.length > 0
			? statusDefs.map(s => ({ value: s.value, label: s.label }))
			: [{ value: '', label: '无' }]
	);
</script>

<select
	class="status-select {status ? 'badge ' + status : ''}"
	value={status}
	onchange={(e) => onchange(e.currentTarget.value)}
	onclick={(e) => e.stopPropagation()}
>
	{#each options as opt}
		<option value={opt.value}>{opt.label}</option>
	{/each}
</select>

<style>
	.status-select {
		border: 1px solid var(--border);
		border-radius: 10px;
		padding: 1px 6px;
		font-size: 12px;
		cursor: pointer;
		background: var(--surface);
	}
</style>

<script lang="ts">
	import { STATUS_OPTIONS } from '$lib/utils/status';
	import type { ItemStatus } from '$lib/types';

	let { status, onchange }: { status: ItemStatus; onchange: (s: ItemStatus) => void } = $props();
</script>

<select
	class="status-select {status ? 'badge ' + status : ''}"
	value={status}
	onchange={(e) => onchange(e.currentTarget.value as ItemStatus)}
	onclick={(e) => e.stopPropagation()}
>
	{#each STATUS_OPTIONS as opt}
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

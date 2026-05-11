<script lang="ts">
	let { value, options, oncommit }: {
		value: string | number | null;
		options: { value: string | number | null; label: string }[];
		oncommit: (val: string | number | null) => void;
	} = $props();

	let editing = $state(false);

	function handleChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		const raw = target.value;
		// Try to parse as number for numeric IDs
		const parsed = raw === '' ? null : isNaN(Number(raw)) ? raw : Number(raw);
		editing = false;
		if (parsed !== value) {
			oncommit(parsed);
		}
	}

	function handleBlur() {
		editing = false;
	}

	const displayLabel = $derived(
		options.find(o => o.value === value)?.label ?? '-'
	);
</script>

{#if editing}
	<!-- svelte-ignore a11y_autofocus -->
	<select
		class="inline-select"
		value={value ?? ''}
		onchange={handleChange}
		onblur={handleBlur}
		autofocus
	>
		{#each options as opt (opt.value)}
			<option value={opt.value ?? ''}>{opt.label}</option>
		{/each}
	</select>
{:else}
	<span
		class="inline-display"
		onclick={() => (editing = true)}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && (editing = true)}
	>
		{displayLabel}
	</span>
{/if}

<style>
	.inline-select {
		border: 1px solid var(--primary);
		border-radius: 3px;
		padding: 1px 4px;
		font-size: inherit;
		font-family: inherit;
	}
	.inline-display {
		cursor: pointer;
		border-bottom: 1px dashed var(--border);
		padding: 0 2px;
	}
	.inline-display:hover {
		border-bottom-color: var(--primary);
		color: var(--primary);
	}
</style>

<script lang="ts">
	let { value, max, suffix = '', oncommit }: {
		value: number;
		max: number;
		suffix?: string;
		oncommit: (val: number) => void;
	} = $props();

	let editing = $state(false);
	let draft = $state('');

	function startEdit() {
		draft = String(value);
		editing = true;
	}

	function commit() {
		editing = false;
		const newVal = Math.max(0, Math.min(max, Number(draft) || 0));
		if (newVal !== value) {
			oncommit(newVal);
		}
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') commit();
		if (e.key === 'Escape') { editing = false; }
	}

	const percent = $derived(Math.min(100, Math.round((value / max) * 100)));
</script>

{#if editing}
	<input
		class="inline-input"
		type="number"
		bind:value={draft}
		min="0"
		{max}
		onblur={commit}
		onkeydown={onKeydown}
		autofocus
	/>
{:else}
	<span
		class="bar-display"
		onclick={startEdit}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && startEdit()}
	>
		{#if value > 0}
			<span class="bar-wrap">
				<span class="bar-fill" style:width="{percent}%"></span>
			</span>
			<span class="bar-value">{value}{suffix}</span>
		{:else}
			<span class="empty">-</span>
		{/if}
	</span>
{/if}

<style>
	.bar-display {
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		flex: 1;
		border-bottom: 1px dashed var(--border);
		padding: 2px 0;
	}
	.bar-display:hover {
		border-bottom-color: var(--primary);
	}
	.bar-wrap {
		flex: 1;
		height: 8px;
		background: var(--bg);
		border-radius: 4px;
		overflow: hidden;
		min-width: 60px;
	}
	.bar-fill {
		display: block;
		height: 100%;
		background: var(--primary);
		border-radius: 4px;
		transition: width 0.2s;
	}
	.bar-value {
		font-size: 12px;
		font-weight: 500;
		color: var(--text);
		white-space: nowrap;
	}
	.empty {
		color: var(--border);
	}
	.inline-input {
		border: 1px solid var(--primary);
		border-radius: 3px;
		padding: 1px 4px;
		font-size: inherit;
		font-family: inherit;
		width: 60px;
	}
</style>

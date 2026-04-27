<script lang="ts">
	let { value, type = 'text', oncommit, min, placeholder = '' }: {
		value: string | number;
		type?: 'text' | 'number';
		oncommit: (val: string | number) => void;
		min?: number;
		placeholder?: string;
	} = $props();

	let editing = $state(false);
	let draft = $state('');

	function startEdit() {
		draft = String(value);
		editing = true;
	}

	function commit() {
		editing = false;
		const newVal = type === 'number' ? Number(draft) : draft;
		if (newVal !== value) {
			oncommit(newVal);
		}
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') commit();
		if (e.key === 'Escape') { editing = false; }
	}
</script>

{#if editing}
	<input
		class="inline-input"
		{type}
		bind:value={draft}
		{min}
		{placeholder}
		onblur={commit}
		onkeydown={onKeydown}
		autofocus
	/>
{:else}
	<span
		class="inline-display"
		onclick={startEdit}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && startEdit()}
	>
		{value || placeholder}
	</span>
{/if}

<style>
	.inline-input {
		border: 1px solid var(--primary);
		border-radius: 3px;
		padding: 1px 4px;
		font-size: inherit;
		font-family: inherit;
		width: auto;
		min-width: 40px;
	}
	.inline-input[type='number'] {
		width: 50px;
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

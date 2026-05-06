<script lang="ts">
	let { value, options, freeform = false, oncommit }: {
		value: string;
		options: string[];
		freeform?: boolean;
		oncommit: (val: string) => void;
	} = $props();

	let editing = $state(false);
	let freeInput = $state('');

	const selected = $derived(value ? value.split(',').filter(Boolean) : []);

	function toggle(opt: string) {
		const set = new Set(selected);
		if (set.has(opt)) set.delete(opt);
		else set.add(opt);
		oncommit([...set].join(','));
	}

	function addFree() {
		const trimmed = freeInput.trim();
		if (!trimmed) return;
		const set = new Set(selected);
		set.add(trimmed);
		freeInput = '';
		oncommit([...set].join(','));
	}

	function handleFreeKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addFree();
		}
	}

	function handleClickOutside(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.pills-editor')) {
			editing = false;
		}
	}
</script>

<svelte:window onclick={handleClickOutside} />

{#if editing}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="pills-editor" onclick={(e) => e.stopPropagation()}>
		<div class="pills-wrap">
			{#each options as opt (opt)}
				<button
					class="pill-btn"
					class:active={selected.includes(opt)}
					onclick={() => toggle(opt)}
				>
					{opt}
				</button>
			{/each}
			{#each selected.filter(s => !options.includes(s)) as custom (custom)}
				<button
					class="pill-btn active custom"
					onclick={() => toggle(custom)}
				>
					{custom} &times;
				</button>
			{/each}
		</div>
		{#if freeform}
			<input
				class="free-input"
				placeholder="自定义..."
				bind:value={freeInput}
				onkeydown={handleFreeKeydown}
			/>
		{/if}
	</div>
{:else}
	<span
		class="inline-display"
		onclick={(e) => { e.stopPropagation(); editing = true; }}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && (editing = true)}
	>
		{#if selected.length > 0}
			{#each selected as s (s)}
				<span class="pill-tag">{s}</span>
			{/each}
		{:else}
			<span class="empty">-</span>
		{/if}
	</span>
{/if}

<style>
	.inline-display {
		cursor: pointer;
		border-bottom: 1px dashed var(--border);
		padding: 0 2px;
		display: inline-flex;
		gap: 4px;
		flex-wrap: wrap;
	}
	.inline-display:hover {
		border-bottom-color: var(--primary);
	}
	.pill-tag {
		font-size: 12px;
		background: var(--bg);
		border: 1px solid var(--border);
		padding: 1px 6px;
		border-radius: 10px;
	}
	.empty {
		color: var(--border);
	}
	.pills-editor {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.pills-wrap {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.pill-btn {
		padding: 2px 10px;
		font-size: 12px;
		border: 1px solid var(--border);
		border-radius: 12px;
		background: var(--surface);
		cursor: pointer;
		transition: all 0.1s;
	}
	.pill-btn:hover {
		border-color: var(--primary);
	}
	.pill-btn.active {
		background: var(--primary);
		color: white;
		border-color: var(--primary);
	}
	.pill-btn.custom {
		background: #6c757d;
		border-color: #6c757d;
	}
	.free-input {
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 2px 6px;
		font-size: 12px;
		width: 100px;
	}
</style>

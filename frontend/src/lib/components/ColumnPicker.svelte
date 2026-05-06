<script lang="ts">
	import { ALL_COLUMNS, saveVisibleColumns } from '$lib/utils/columns';

	let { visibleKeys = $bindable<string[]>() }: {
		visibleKeys: string[];
	} = $props();

	let open = $state(false);

	function toggle(key: string) {
		if (key === 'name') return; // name is always visible
		if (visibleKeys.includes(key)) {
			visibleKeys = visibleKeys.filter((k) => k !== key);
		} else {
			visibleKeys = [...visibleKeys, key];
		}
		saveVisibleColumns(visibleKeys);
	}

	function handleClickOutside(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.column-picker')) {
			open = false;
		}
	}
</script>

<svelte:window onclick={handleClickOutside} />

<div class="column-picker">
	<button
		class="picker-btn"
		title="选择显示列"
		onclick={(e) => { e.stopPropagation(); open = !open; }}
	>
		⚙
	</button>

	{#if open}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="picker-dropdown" onclick={(e) => e.stopPropagation()}>
			<div class="picker-title">显示列</div>
			{#each ALL_COLUMNS as col (col.key)}
				<label class="picker-item">
					<input
						type="checkbox"
						checked={visibleKeys.includes(col.key)}
						disabled={col.key === 'name'}
						onchange={() => toggle(col.key)}
					/>
					{col.label}
				</label>
			{/each}
		</div>
	{/if}
</div>

<style>
	.column-picker {
		position: relative;
	}

	.picker-btn {
		padding: 4px 8px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: var(--surface);
		cursor: pointer;
		font-size: 16px;
		line-height: 1;
		color: var(--text-secondary);
	}
	.picker-btn:hover {
		background: var(--bg);
		color: var(--text);
	}

	.picker-dropdown {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: 4px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 8px 0;
		min-width: 160px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		z-index: 100;
	}

	.picker-title {
		padding: 4px 12px 8px;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		border-bottom: 1px solid var(--border);
		margin-bottom: 4px;
	}

	.picker-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 12px;
		font-size: 13px;
		cursor: pointer;
	}
	.picker-item:hover {
		background: var(--bg);
	}
	.picker-item input[type='checkbox'] {
		width: auto;
	}
	.picker-item input:disabled {
		opacity: 0.5;
	}
</style>

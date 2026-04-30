<script lang="ts">
	import type { Snippet } from 'svelte';
	import StickyPanel from './StickyPanel.svelte';

	let { left, right }: {
		left: Snippet;
		right: Snippet;
	} = $props();

	let activeTab = $state<'checklist' | 'inventory'>('checklist');
	let rightCollapsed = $state(false);
</script>

<!-- Mobile tabs -->
<div class="mobile-tabs">
	<button
		class="tab-btn"
		class:active={activeTab === 'checklist'}
		onclick={() => (activeTab = 'checklist')}
	>
		清单
	</button>
	<button
		class="tab-btn"
		class:active={activeTab === 'inventory'}
		onclick={() => (activeTab = 'inventory')}
	>
		物品库
	</button>
</div>

<div class="split-pane" class:collapsed={rightCollapsed}>
	<div class="left-panel" class:mobile-hidden={activeTab !== 'checklist'}>
		{@render left()}
	</div>

	<!-- Desktop toggle button -->
	<button
		class="collapse-toggle"
		onclick={() => (rightCollapsed = !rightCollapsed)}
		title={rightCollapsed ? '展开物品库' : '收起物品库'}
	>
		{rightCollapsed ? '◀ 物品库' : '▶'}
	</button>

	{#if !rightCollapsed}
		<div class="right-panel" class:mobile-hidden={activeTab !== 'inventory'}>
			<StickyPanel>
				{@render right()}
			</StickyPanel>
		</div>
	{/if}
</div>

<style>
	.mobile-tabs {
		display: none;
		gap: 0;
		margin-bottom: 12px;
		border: 1px solid var(--border);
		border-radius: 6px;
		overflow: hidden;
	}
	.tab-btn {
		flex: 1;
		padding: 8px;
		border: none;
		border-radius: 0;
		font-size: 14px;
		font-weight: 500;
		background: var(--surface);
		cursor: pointer;
	}
	.tab-btn.active {
		background: var(--primary);
		color: white;
	}
	.split-pane {
		display: flex;
		gap: 16px;
		align-items: flex-start;
	}
	.left-panel {
		flex: 55;
		min-width: 0;
	}
	.split-pane.collapsed .left-panel {
		flex: 1;
	}
	.collapse-toggle {
		display: block;
		flex-shrink: 0;
		padding: 8px 6px;
		border: 1px solid var(--border);
		border-radius: 6px;
		background: var(--surface);
		cursor: pointer;
		font-size: 12px;
		color: var(--text-secondary);
		writing-mode: vertical-lr;
		position: sticky;
		top: 16px;
		align-self: flex-start;
	}
	.collapse-toggle:hover {
		background: var(--bg);
		color: var(--text);
	}
	.right-panel {
		flex: 45;
		min-width: 0;
		border-radius: 12px;
		align-self: stretch;
	}

	@media (max-width: 768px) {
		.mobile-tabs {
			display: flex;
		}
		.collapse-toggle {
			display: none;
		}
		.split-pane {
			flex-direction: column;
		}
		.left-panel,
		.right-panel {
			flex: 1;
			width: 100%;
		}
		.right-panel :global(.sticky-panel) {
			position: static;
			max-height: none;
			overflow-y: visible;
		}
		.mobile-hidden {
			display: none;
		}
	}
</style>
